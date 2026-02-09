import { env, file, sleep, spawn } from "bun";

function execute(code: string, countMax: number): boolean {
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
    for (let _ = 0; _ < countMax; _++) {
        if (pc >= code.length) return true;
        if (pointer < 0 || pointer > 65535) { // Out of range
            return true;
        }
        switch (code[pc]) {
            case "+": memory[pointer]!++; break;
            case "-": memory[pointer]!--; break;
            case "<": pointer--; break;
            case ">": pointer++; break;
            case "[": if (memory[pointer] == 0) { pc = jmpMap.get(pc)!; } break;
            case "]": if (memory[pointer] != 0) { pc = jmpMap.get(pc)!; } break;
        }
        pc++;
    }
    return false;
}

function _randombf(max: number) {
    let lv = 0;
    let code = "";
    while (lv >= 0) {
        if (code.length > max) {
            code = "";
            lv = 0;
        }
        switch (Math.floor(Math.random() * 7)) {
            case 0: code += "+"; break;
            case 1: code += "-"; break;
            case 2: code += "<"; break;
            case 3: code += ">"; break;
            case 4: code += "["; lv += 1; break;
            case 5: code += "]"; lv -= 1; break;
            case 6: code += "."; break;
        }
    }
    return code.substring(0, code.length - 1);
}
function GenerateRandomBFCode(min: number, max: number): string {
    let code = "";
    while ((code = _randombf(max)).length < min);
    return code;
}
const QBF_FILE = `target/${env.QBF_MODE ?? "debug"}/qbf`;
const TEMP_BF = "./box/bf/temp.bf";
const tmp = file(TEMP_BF);
while (true) {
    const code = GenerateRandomBFCode(100, 1000);
    if (execute(code, 100000)) {
        await tmp.write(code);
        const qbf_process = spawn({
            cmd: [QBF_FILE, TEMP_BF],
        });
        switch (await Promise.race([qbf_process.exited, sleep(1000)])) {
            case undefined:
                qbf_process.kill();
                console.error("Timeout");
                process.exit();
            case 0: // Expected behavior in fuzzing
                break;
            default: // Case of panic
                process.exit();
        }
    }
}
