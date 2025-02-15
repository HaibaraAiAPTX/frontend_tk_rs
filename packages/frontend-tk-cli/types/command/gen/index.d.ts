interface GenOps {
    input?: string;
    plugin?: string;
    modelOutput?: string[];
    serviceOutput?: string[];
    serviceMode?: string[];
}
export declare function gen(ops: GenOps): Promise<void>;
export {};
