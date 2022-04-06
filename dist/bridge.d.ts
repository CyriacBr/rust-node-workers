declare type Task = <T>(payload?: T) => object | null;
declare type Tasks = Record<string, Task>;
interface Options {
    debug?: boolean;
}
export declare function bridge(tasks: Tasks, opts?: Options): void;
export {};
//# sourceMappingURL=bridge.d.ts.map