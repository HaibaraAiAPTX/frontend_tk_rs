// ============================================
// IR input types (primary use for plugins)
// ============================================

export interface GeneratorInput {
  project: ProjectContext;
  endpoints: EndpointItem[];
  model_import: ModelImportConfig | null;
  client_import: ClientImportConfig | null;
  output_root: string | null;
}

export interface EndpointItem {
  namespace: string[];
  operation_name: string;
  export_name: string;
  builder_name: string;
  summary?: string;
  method: string;
  path: string;
  input_type_name: string;
  output_type_name: string;
  request_body_field?: string;
  query_fields: string[];
  path_fields: string[];
  has_request_options: boolean;
  deprecated: boolean;
  meta: Record<string, string>;
}

export interface ProjectContext {
  package_name: string;
  api_base_path?: string;
  terminals: string[];
  retry_ownership?: string;
}

export interface ModelImportConfig {
  import_type: string;
  package_path?: string;
  relative_path?: string;
  original_path?: string;
}

export interface ClientImportConfig {
  mode: string;
  client_path?: string;
  client_package?: string;
  import_name?: string;
}

// ============================================
// Pipeline execution types (for advanced plugins)
// ============================================

export interface PlannedFile {
  path: string;
  content: string;
}

export interface RendererExecution {
  renderer_id: string;
  planned_files: number;
  warnings: string[];
}

export interface ExecutionPlan {
  endpoint_count: number;
  transform_steps: string[];
  renderer_reports: RendererExecution[];
  planned_files: PlannedFile[];
  skipped_files: number;
  metrics: ExecutionMetrics;
}

export interface ExecutionMetrics {
  parse_ms: number;
  transform_ms: number;
  render_ms: number;
  layout_ms: number;
  write_ms: number;
  total_ms: number;
}
