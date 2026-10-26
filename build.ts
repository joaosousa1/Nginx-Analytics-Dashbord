import { readdirSync, unlinkSync, writeFileSync, existsSync, renameSync, readFileSync } from "node:fs";
import { join } from "node:path";

const hash = Date.now().toString(36); // Generate unique hash

// 1. Clean up old files in static/css and static/js
const dirsToClean = ["./static/js", "./static/css"];
for (const dir of dirsToClean) {
  if (existsSync(dir)) {
    try {
      const files = readdirSync(dir);
      for (const file of files) {
        if (file.endsWith(".min.css") || file.endsWith(".min.js") || file.endsWith(".css") || file.endsWith(".js")) {
          const filePath = join(dir, file);
          console.log(`Deleting old asset: ${filePath}`);
          unlinkSync(filePath);
        }
      }
    } catch (e) {
      console.error(`Error cleaning directory ${dir}:`, e);
    }
  }
}

console.log("Building assets with Bun...");

// 2. Build CSS strictly into ./static/css
const cssResult = await Bun.build({
  entrypoints: ["./assets/css/style.css"],
  minify: true,
  outdir: "./static/css",
});

if (!cssResult.success) {
  console.error("CSS Build failed");
  process.exit(1);
}

// Rename generated CSS to include the hash
const cssFiles = readdirSync("./static/css");
for (const file of cssFiles) {
  if (file.endsWith(".css") && !file.includes(hash)) {
    renameSync(join("./static/css", file), join("./static/css", `style.min.${hash}.css`));
  }
}

// 3. Build JS strictly into ./static/js
const jsResult = await Bun.build({
  entrypoints: ["./assets/js/app.js"],
  minify: true,
  outdir: "./static/js",
  plugins: [],
});

if (!jsResult.success) {
  console.error("JS Build failed");
  process.exit(1);
}

// Clean up any accidental CSS files generated inside ./static/js and rename JS
const jsFiles = readdirSync("./static/js");
for (const file of jsFiles) {
  if (file.endsWith(".css")) {
    unlinkSync(join("./static/js", file));
  } else if (file.endsWith(".js") && !file.includes(hash)) {
    renameSync(join("./static/js", file), join("./static/js", `app.min.${hash}.js`));
  }
}

// 4. Update .env file safely, preserving USER, PASS, and NGINX_LOG
const envPath = ".env";
let envContent = "";

if (existsSync(envPath)) {
  envContent = readFileSync(envPath, "utf-8");
} else {
  envContent = "USER=admin\nPASS=supersecret\nNGINX_LOG=/var/log/nginx/access.log\n";
}

function upsertEnv(content: string, key: string, value: string): string {
  const regex = new RegExp(`^${key}=.*`, "m");
  if (regex.test(content)) {
    return content.replace(regex, `${key}=${value}`);
  } else {
    return content.trimEnd() + `\n${key}=${value}\n`;
  }
}

const cssPathValue = `/static/css/style.min.${hash}.css`;
const jsPathValue = `/static/js/app.min.${hash}.js`;

envContent = upsertEnv(envContent, "CSS_PATH", cssPathValue);
envContent = upsertEnv(envContent, "JS_PATH", jsPathValue);

writeFileSync(envPath, envContent.trim() + "\n", "utf-8");

console.log(`.env updated successfully with hash: ${hash}`);
console.log("Build completed successfully!");