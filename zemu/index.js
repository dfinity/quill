import z, { DEFAULT_START_OPTIONS } from "@zondax/zemu";
import process from "node:process";
import path from "node:path";
import fs from "node:fs/promises"
/** @type {typeof z} */
const Zemu = z.default;

const sim = new Zemu(path.resolve("app.elf"));
async function shutdown() {
    await sim.close();
    process.exit(0);
}
process.on('SIGINT', shutdown);
process.stdin.on('close', shutdown);
try {
    await sim.start({
        ...DEFAULT_START_OPTIONS,
        model: "nanosp",
        startText: "Internet",
        custom: "-s \"equip will roof matter pink blind book anxiety banner elbow sun young\""
    });
    sim.startGRPCServer("127.0.0.1", 44223);
    await new Promise(rs => setTimeout(rs, 1000)); /* 👽 */
    await fs.writeFile(process.argv[2], "44223\n");
    while (true) {
        await sim.waitUntilScreenIsNot(sim.getMainMenuSnapshot());
        await sim.navigateUntilText(".", "cargo-test", "APPROVE");
        await sim.waitUntilScreenIs(sim.getMainMenuSnapshot());
    }
} catch (e) {
    await sim.close();
    throw e;
}
