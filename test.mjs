import { transform } from "@swc/core";

const input = `import packages from "./package.json?raw";`;
const output = `const packages = "MIT License...";`;

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
