use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;
use std::str::FromStr;
use swagger_gen::pipeline::{
    AxiosTsRenderer, NormalizeEndpointPass, Renderer, TransformPass, build_dry_run_plan,
    parse_openapi_to_ir,
};
use swagger_tk::model::OpenAPIObject;

fn build_openapi_text(endpoint_count: usize) -> String {
    let mut paths = serde_json::Map::new();

    for i in 1..=endpoint_count {
        let path = format!("/bench/v1/resource-{i}/{{id}}");
        let group = format!("bench-group-{}", (i % 20) + 1);
        let operation = format!("benchGetResource{i}");

        paths.insert(
            path,
            json!({
              "get": {
                "tags": [group],
                "summary": format!("Benchmark API {i}"),
                "operationId": operation,
                "parameters": [
                  {
                    "name": "id",
                    "in": "path",
                    "required": true,
                    "schema": { "type": "string" }
                  },
                  {
                    "name": "page",
                    "in": "query",
                    "required": false,
                    "schema": { "type": "integer", "format": "int32" }
                  }
                ],
                "responses": {
                  "200": {
                    "description": "ok",
                    "content": {
                      "application/json": {
                        "schema": { "$ref": "#/components/schemas/BenchItem" }
                      }
                    }
                  }
                }
              }
            }),
        );
    }

    serde_json::to_string(&json!({
      "openapi": "3.1.0",
      "info": {
        "title": "Aptx Benchmark 10k API",
        "version": "1.0.0"
      },
      "paths": paths,
      "components": {
        "schemas": {
          "BenchItem": {
            "type": "object",
            "properties": {
              "id": { "type": "string" },
              "value": { "type": "string" },
              "count": { "type": "integer", "format": "int32" }
            },
            "required": ["id", "value"]
          }
        }
      }
    }))
    .expect("serialize benchmark openapi")
}

fn benchmark_codegen_pipeline(c: &mut Criterion) {
    let openapi_text = build_openapi_text(10_000);
    let openapi = OpenAPIObject::from_str(&openapi_text).expect("parse OpenAPI sample");

    let mut group = c.benchmark_group("codegen_pipeline_10k");
    group.sample_size(10);

    group.bench_function("parse_to_ir", |b| {
        b.iter(|| {
            parse_openapi_to_ir(black_box(&openapi)).expect("parse_to_ir");
        })
    });

    group.bench_function("dry_plan", |b| {
        b.iter(|| {
            build_dry_run_plan(black_box(&openapi)).expect("dry_plan");
        })
    });

    let mut ir = parse_openapi_to_ir(&openapi).expect("build ir once");
    NormalizeEndpointPass
        .apply(&mut ir)
        .expect("normalize endpoint names");
    let renderer = AxiosTsRenderer;

    group.bench_function("axios_ts_render", |b| {
        b.iter(|| {
            renderer.render(black_box(&ir)).expect("axios_ts_render");
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_codegen_pipeline);
criterion_main!(benches);
