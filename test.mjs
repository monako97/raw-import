import { transform } from "@swc/core";
import fs from "fs";

const input = `import packages from "./package.json?raw";`;
const output = "var packages = '" + JSON.stringify(JSON.parse(fs.readFileSync("./package.json", "utf-8")), null, 4).replaceAll(/\n/g, '\\n').replace(/^"/, "").replace(/"$/, "") + "\\n';";

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
      console.log("Expected Output:\n", output.trim(), output.trim().length);
      console.log("Actual Output:\n", code?.trim(), code?.trim().length);
      throw new Error("Test failed: Output did not match expected output");
    }
  })
  .catch((err) => {
    throw err;
  });
