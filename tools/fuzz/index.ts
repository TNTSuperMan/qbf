import { env, file, sleep, spawn } from "bun";
import { execute } from "../exec/core";

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
    const exec_result = execute({ code, timeout_cycles: 100_000 });
    if (exec_result.type !== "timeout") {
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
