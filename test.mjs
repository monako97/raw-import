import { transform } from "@swc/core";
import fs from "fs";

const input = fs.readFileSync("./tests/input.js", "utf-8");
const output = "var packages = '" + JSON.stringify(JSON.parse(fs.readFileSync("./package.json", "utf-8")), null, 3).replaceAll(/\n/g, '\\n').replace(/^"/, "").replace(/"$/, "") + "\\n;";

const cwd = process.cwd();

console.log(cwd)
transform(input, {
  jsc: {
    experimental: {
      plugins: [
        [
          "./raw_import.wasm",
          {
            rootDir: process.cwd(),
          },
        ],
      ],
    },
  },
})
  .then(({ code }) => {
    if (code?.trim() === output.trim()) {
      console.log("Test passed!");
    } else {
      console.log("Expected Output:\n", output);
      console.log("Actual Output:\n", code);
      throw new Error("Test failed: Output did not match expected output");
    }
  })
  .catch((err) => {
    throw err;
  });
