[main]
micro main(): i32 {
    let i = 0;
    loop i < 5 {
        i = i + 1;
        if i == 3 {
            i = i + 10;
        }
    }
    return i;
}
