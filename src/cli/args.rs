pub(super) fn argv(args: impl IntoIterator<Item = String>) -> Vec<String> {
    let args = args.into_iter().collect::<Vec<_>>();
    let args = match args.as_slice() {
        #[cfg(debug_assertions)]
        [arg] if arg == "--probe" || arg == "--once" => vec!["probe".to_owned()],
        #[cfg(debug_assertions)]
        [arg] if arg == "--toast-test" => vec!["toast-test".to_owned()],
        _ => args,
    };

    std::iter::once("xbattery".to_owned()).chain(args).collect()
}
