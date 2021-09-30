[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line(message: utf16): unit;

[clr("mscorlib", "System.Environment", "GetCommandLineArgs")]
micro get_args(): [utf16];

[main]
micro main(args: [utf16]): i32 {
    let i: i32 = 0;
    var has_build: bool = false;
    while i < args.len() {
        if args[i] == "build" {
            has_build = true;
        }
        i = i + 1;
    }
    if has_build {
        console_write_line("bootstrap build phase");
        return 0;
    }
    else {
        return 0;
    }
}
