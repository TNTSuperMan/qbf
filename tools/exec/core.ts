import { nanoseconds } from "bun";

export type ExecResult =
    | { type: "timeout" }
    | { type: "outofrange", pointer: number }
    | { type: "success", cycles: number, time_ns: number };
export interface ExecOptions {
    code: string;
    timeout_cycles?: number;
    output?: (value: number) => void;
    input?: () => number;
}

export function execute({ code, timeout_cycles, output, input }: ExecOptions): ExecResult {
    const start = nanoseconds();
    let pc = 0;
    let pointer = 0;
    const memory = new Uint8Array(65536);
    const jmpMap = new Map<number, number>();
    const stack: number[] = [];
    for (let i = 0; i < code.length; i++) {
        if (code[i] == "[") {
            stack.push(i);
        } else if (code[i] == "]") {
            const start = stack.pop()!;
            jmpMap.set(start, i);
            jmpMap.set(i, start);
        }
    }
    const timeout_ = timeout_cycles ?? Infinity;
    for (let cycles = 0; cycles < timeout_; cycles++) {
        if (pc >= code.length) {
            const end = nanoseconds();
            return { type: "success", cycles: cycles, time_ns: end - start };
        }
        if (pointer < 0 || pointer > 65535) { // Out of range
            return { type: "outofrange", pointer };
        }
        switch (code[pc]) {
            case "+": memory[pointer]!++; break;
            case "-": memory[pointer]!--; break;
            case "<": pointer--; break;
            case ">": pointer++; break;
            case "[": if (memory[pointer] == 0) { pc = jmpMap.get(pc)!; } break;
            case "]": if (memory[pointer] != 0) { pc = jmpMap.get(pc)!; } break;
            case ".": output?.(memory[pointer]!); break;
            case ",": memory[pointer] = input?.() ?? 0; break;
        }
        pc++;
    }
    return { type: "timeout" };
}
