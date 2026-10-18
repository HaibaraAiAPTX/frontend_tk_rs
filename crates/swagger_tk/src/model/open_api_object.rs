use super::{ComponentsObject, OpenAPIWebhooks};
use super::{ExternalDocumentationObject, OpenAPIInfo, PathItemObject, ServerObject, TagObject};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAPIObject {
    /// 版本号
    pub openapi: String,

    /// 信息
    pub info: Option<OpenAPIInfo>,

    // pub json_schema_dialect: String,
    pub servers: Option<Vec<ServerObject>>,

    pub paths: Option<HashMap<String, PathItemObject>>,

    pub webhooks: Option<HashMap<String, OpenAPIWebhooks>>,

    pub components: Option<ComponentsObject>,

    pub security: Option<Vec<HashMap<String, Vec<String>>>>,

    pub tags: Option<Vec<TagObject>>,

    #[serde(rename = "externalDocs")]
    pub external_docs: Option<ExternalDocumentationObject>,
}

/// 解析整份 OpenAPI 文档时的错误。
///
/// 拆成两类是因为两者的可定位性来源完全不同：
///
/// - [`Syntax`](OpenAPIParseError::Syntax)：JSON 本身不合法，位置就是原文的 `line / column`；
/// - [`Schema`](OpenAPIParseError::Schema)：JSON 合法但结构不符合本 crate 的模型，
///   位置由 `serde_path_to_error` 给出的**字段路径**表示（例如
///   `components.schemas.Order`）。`aptx-root/swagger/*.json` 是单行压缩 JSON，一份 spec
///   只有两行、column 能到 17000+，`line / column` 对排查毫无用处，字段路径才是可定位的坐标。
#[derive(Debug)]
pub enum OpenAPIParseError {
    /// JSON 语法错误，位置来自原文行列号。
    Syntax(serde_json::Error),
    /// JSON 合法但反序列化失败，`path` 是失败位置的字段路径（`Path::empty` 表示根节点）。
    Schema {
        path: serde_path_to_error::Path,
        source: serde_json::Error,
    },
}

impl fmt::Display for OpenAPIParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // 语法错误原样透传：`serde_json` 自带的 `at line N column M` 就是最准的位置。
            OpenAPIParseError::Syntax(err) => write!(f, "{err}"),
            OpenAPIParseError::Schema { path, source } => match format_path(path) {
                // 错误就在根节点上，补一个空路径只是噪音。
                None => write!(f, "{source}"),
                Some(path) => write!(f, "{path}: {source}"),
            },
        }
    }
}

impl std::error::Error for OpenAPIParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OpenAPIParseError::Syntax(err) => Some(err),
            OpenAPIParseError::Schema { source, .. } => Some(source),
        }
    }
}

impl OpenAPIParseError {
    /// 失败位置对应的字段路径；语法错误或根节点错误时为空字符串。
    ///
    /// 消费方（CLI / 插件）想自己拼前缀时用这个，不要反过来解析 [`Display`] 的文本。
    pub fn path(&self) -> String {
        match self {
            OpenAPIParseError::Syntax(_) => String::new(),
            OpenAPIParseError::Schema { path, .. } => format_path(path).unwrap_or_default(),
        }
    }
}

impl From<serde_json::Error> for OpenAPIParseError {
    fn from(err: serde_json::Error) -> Self {
        OpenAPIParseError::Syntax(err)
    }
}

/// 把路径渲染成 `components.schemas.Order` 形式；空路径 / 根路径返回 `None`。
///
/// `serde_path_to_error` 用 `.` 表示「根节点」，直接拼进报错会变成 `.: ...`。
fn format_path(path: &serde_path_to_error::Path) -> Option<String> {
    let path = path.to_string();
    if path.is_empty() || path == "." {
        None
    } else {
        Some(path)
    }
}

/// 错误是否属于「JSON 文本本身有问题」，而不是「JSON 合法但结构不符合模型」。
///
/// `Category::Syntax` 之外还必须把 `Category::Eof` 算进来：截断的输入（例如
/// `{"info": {"title":"t"`）分类是 `Eof`，此时 `serde_path_to_error` 已经走过
/// `info`，会留下 `info.?` 这种带 `?` 的半截路径。对「文件没写完」来说，
/// 原文的 `line / column` 才是唯一有意义的坐标。
fn is_json_level_error(err: &serde_json::Error) -> bool {
    matches!(
        err.classify(),
        serde_json::error::Category::Syntax | serde_json::error::Category::Eof
    )
}

impl FromStr for OpenAPIObject {
    type Err = OpenAPIParseError;

    fn from_str(swagger_text: &str) -> Result<Self, Self::Err> {
        // 这里的 `serde_path_to_error` 包装是「报错可定位」的全部来源，**不要**退回
        // 裸 `serde_json::from_str`：它只会在最外层报 `data did not match ... at line 2
        // column 61`，而真实 spec 是单行压缩 JSON，行列号没有任何指向性。
        //
        // 它是单趟解析（不像「先 Value 再强类型」那样把文档解析两遍）：路径由被包装的
        // `Deserializer` 在每次进出字段时记录，语法错误则原样带着行列号返回。
        let mut json = serde_json::Deserializer::from_str(swagger_text);
        let mut track = serde_path_to_error::Track::new();
        let tracked = serde_path_to_error::Deserializer::new(&mut json, &mut track);

        let open_api_object = match OpenAPIObject::deserialize(tracked) {
            Ok(open_api_object) => open_api_object,
            // 语法 / EOF 错误自带行列号，不需要再挂路径：此时路径停在出错前的最后一个字段上
            // （截断的输入会得到 `info.?` 这种带 `?` 的半截路径），加上反而误导。
            Err(source) if is_json_level_error(&source) => {
                return Err(OpenAPIParseError::Syntax(source));
            }
            Err(source) => {
                return Err(OpenAPIParseError::Schema {
                    path: track.path(),
                    source,
                });
            }
        };

        // `serde_json::from_str` 会拒绝文档末尾的多余内容，这里必须手动补上，
        // 否则 `{...}{...}` 这类输入会被静默接受。
        json.end()?;

        Ok(open_api_object)
    }
}
