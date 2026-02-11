import { env, sleep, spawn, stdout, write } from "bun";
import { execute } from "../exec/core";
import { GenerateRandomBFCode } from "../rand/core";

const MAX_STEPS = 1000;
const TIMEOUT_MS = 100;
const INPUT_MINLEN = 100;
const INPUT_MAXLEN = 500;
const PROB: [string, number][] = [
    ["[", 3],
    ["]", 3],
    ["<", 2],
    [">", 2],
    ["+", 1],
    ["-", 1],
    [".", 1],
];

const QBF_FILE = `target/${env.QBF_MODE ?? "debug"}/brainrot`;

async function report(code: string, description: string) {
    console.error("", description);
    await write(`./box/fuzz/${crypto.randomUUID()}.bf`, `[
${description.replaceAll(`[`,`{`).replaceAll(']','}')}
]
${code}`);
}

while (true) {
    const code = GenerateRandomBFCode(INPUT_MINLEN, INPUT_MAXLEN);
    const exec_result = execute({ code, timeout_cycles: MAX_STEPS });
    if (exec_result.type !== "timeout") {
        const brainrot_process = spawn({
            cmd: [QBF_FILE, "/dev/stdin"],
            stdin: new Blob([code]),
            stderr: "pipe",
        });
        const race_res = await Promise.race([brainrot_process.exited, sleep(TIMEOUT_MS)]);
        if (race_res === undefined) { // sleep(TIMEOUT_MS)の挙動
            brainrot_process.kill();
            stdout.write("_");
            //await report(code, "timeout");
        } else {
            stdout.write(".");
            const stderr = await brainrot_process.stderr.text();
            switch (race_res) {
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
}
