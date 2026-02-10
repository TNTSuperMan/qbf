import { argv, file, nanoseconds, spawn } from "bun";

const [,, count, execfile, sourcefile, inputfile] = argv;

if (!sourcefile) {
    console.log("usage: bun tools/exec [count] [execfile] [sourcefile] [input?]");
} else {
    const c = Number(count);
    const stdin = new Blob(inputfile ? [await file(inputfile).bytes()] : []);
    
    async function exec() {
        const task = spawn({
            cmd: [execfile!, sourcefile!],
            stdout: "ignore",
            stdin,
        });
        if ((await task.exited) !== 0) {
            console.error("not success");
            process.exit();
        }
    }

    console.log("warm up");
    for (let i = 0; i < c; i++) {
        await exec();
        console.log(`${i+1}/${c}`);
    }

    const times: number[] = [];
    for (let i = 0; i < c; i++) {
        const start = nanoseconds();
        await exec();
        const end = nanoseconds();
        console.log(`${i+1}/${c}: ${end-start}ns`);
        times.push(end - start);
    }

    console.log(`avg: ${times.reduce((p,v)=>p+v,0)/times.length}ns`);
}
