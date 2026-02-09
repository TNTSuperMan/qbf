import { argv, file, stdout } from "bun";
import { execute } from "./core";

const [,, sourcefile, timeout] = argv;

if (!sourcefile) {
    console.log("usage: bun tools/exec [sourcefile] [timeout?]");
} else {
    let timeout_num = Number(timeout ?? "NaN");
    if (isNaN(timeout_num)) timeout_num = Infinity;
    const result = execute({
        code: await file(sourcefile).text(),
        timeout_cycles: timeout_num,
        output(v) { stdout.write(String.fromCharCode(v)); },
    });
    switch (result.type) {
        case "timeout": console.log("Result: Timeout"); break;
        case "outofrange": console.log(`Result: Out of range to ${result.pointer}`); break;
        case "success": console.log(`Result: Success in ${result.time_ns}ns, ${result.cycles} cycles`);
    }
}
