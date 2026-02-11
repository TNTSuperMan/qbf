import { argv } from "bun";
import { GenerateRandomBFCode } from "./core";

const [,, min_str, max_str] = argv;

let min = Number(min_str);
if (isNaN(min)) min = 100;
let max = Number(max_str);
if (isNaN(max)) min = 500;

console.log(GenerateRandomBFCode(min, max));
