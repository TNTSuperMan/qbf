export function GenerateRandomBFCode(min: number, max: number) {
    let lv = 0;
    let code = "";
    while (lv >= 0 || code.length < min) {
        if (code.length > (max-lv)) {
            code += "]".repeat(lv);
            return code;
        }
        switch (Math.floor(Math.random() * 7)) {
            case 0: code += "+"; break;
            case 1: code += "-"; break;
            case 2: code += "<"; break;
            case 3: code += ">"; break;
            case 4: code += "["; lv += 1; break;
            case 5: if (lv >= 1) { code += "]"; lv -= 1; } break;
            case 6: code += "."; break;
        }
    }
    return code.substring(0, code.length - 1);
}
