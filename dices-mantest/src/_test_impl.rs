use dices_man::example::CodeExample;

use dices_engine::Engine;

/// Main testing function
pub(crate) fn test_inner(test: &str, _tags: &[&str]) {
    // Parse the test
    let test: CodeExample = test.parse().expect("The test should be parseable");
    // Create the engine
    let mut engine: Engine<rand_xoshiro::Xoshiro256PlusPlus, _> = Engine::new();
    // run the test
    for (n, piece) in test.iter().enumerate() {
        let res = engine
            .eval_multiple(&piece.cmd.command)
            .expect("Error in the execution of the doctest!");
        if let Some(checker) = piece.res.as_ref() {
            assert!(
                checker.is_match(&res),
                "The result number {} was {}, not satisfing the matcher",
                n + 1,
                res
            )
        }
    }
}
