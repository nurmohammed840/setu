export async function runCmd(program: string, args: string[] = []) {
    const cmd = new Deno.Command(program, {
        args,
        stdout: "inherit",
        stderr: "inherit",
        stdin: "null",
    });

    const child = cmd.spawn();
    const status = await child.status;

    return status.code;
}
