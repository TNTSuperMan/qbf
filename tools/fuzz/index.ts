import { env, file, sleep, spawn, stdout, write } from "bun";
import { execute } from "../exec/core";

const MAX_STEPS = 1000;
const TIMEOUT_MS = 100;
const INPUT_MINLEN = 100;
const INPUT_MAXLEN = 500;

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
const QBF_FILE = `target/${env.QBF_MODE ?? "debug"}/brainrot`;

async function report(code: string, description: string) {
    console.error("", description);
    await write(`./box/fuzz/${crypto.randomUUID()}.bf`, `[
${description.replaceAll(`[`,`{`).replaceAll(']','}')}
]
${code}`)
}

while (true) {
    const code = GenerateRandomBFCode(INPUT_MINLEN, INPUT_MAXLEN);
    const exec_result = execute({ code, timeout_cycles: MAX_STEPS });
    if (exec_result.type !== "timeout") {
        stdout.write(".");
        const brainrot_process = spawn({
            cmd: [QBF_FILE, "/dev/stdin"],
            stdin: new Blob([code]),
            stderr: "pipe",
        });
        const race_res = await Promise.race([brainrot_process.exited, sleep(TIMEOUT_MS)]);
        const stderr = await brainrot_process.stderr.text();
        if (!race_res) { // sleep(TIMEOUT_MS)の挙動
            brainrot_process.kill();
            //await report(code, "timeout");
        } else switch (race_res) {
            case 0: // Expected behavior in fuzzing
                break;
            case 101: // panic
                await report(code, `panic occurred:\n${stderr}`);
                break;
            default:
                await report(code, `unknown behavior, exitcode: ${race_res}. stderr:\n${stderr}`);
                break;
        }
    }
}
