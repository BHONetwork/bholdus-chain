import { program } from "commander";
import * as fs from "fs";
import * as path from "path";
import klawSync from "klaw-sync";

program
  .name("set-version")
  .description("CLI to set all bholdus packages version in Cargo.toml");

program.argument("<version>", "Version to be set");
program.argument("<path>", "Relative path to the bholdus repository");

program.parse();

const version = program.args[0];
const repoPath = path.resolve(__dirname, program.args[1]);

console.log(`The bholdus repository path: ${repoPath}`);

if (!fs.existsSync(repoPath)) {
  console.error("The bholdus repository not exists");
}

// Ignore some dirs
// TODO: We should use .gitignore to get ignored dirs.
const ignorePaths = ["target", "node_modules"];

const paths = klawSync(repoPath, {
  nodir: true,
  filter: (item) => {
    const stat = fs.statSync(item.path);

    if (stat.isFile()) {
      const fileName = path.basename(item.path);
      // console.log('filename', fileName);
      if (fileName === "Cargo.toml") {
        return true;
      }
      return false;
    } else if (stat.isDirectory()) {
      const shouldIgnore = ignorePaths.some((ignoredPath) => {
        if (item.path.includes(ignoredPath)) {
          return true;
        }
      });

      if (shouldIgnore) {
        return false;
      }

      const dirName = path.basename(item.path);

      // Ignore hidden dirs
      if (dirName === "." || dirName[0] !== ".") {
        return true;
      }

      return false;
    }
  },
});

const regexp = /^version +?= +?['"](.+?)['"]$/m;

for (const path of paths) {
  const fileContent = fs.readFileSync(path.path, "utf-8");
  const newContent = fileContent.replace(regexp, `version = "${version}"`);
  fs.writeFileSync(path.path, newContent, "utf8");
  console.log(`Update version ${version} to ${path.path}`);
}
