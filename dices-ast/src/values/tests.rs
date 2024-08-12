mod roundtrips {
    use super::super::*;

    fn check_roundtrip(value: Value) {
        let serialized = dbg!(value.to_string());
        let reparsed: Value = serialized.parse().expect("The value should be parseable");
        assert_eq!(
            value, reparsed,
            "The value parsed is different from the original"
        )
    }

    macro_rules! roundtrips {
        (
            $(
                $name:ident: $value:expr ;
            )*
        ) => {
        $(
            #[test]
            fn $name() {
                check_roundtrip($value.into())
            }
        )*};
    }
    roundtrips! {
        null: ValueNull;
        bool_true:  ValueBool::TRUE;
        bool_false: ValueBool::FALSE;
        num_zero: ValueNumber::from(0);
        num_one: ValueNumber::from(1);
        num_minus_one: ValueNumber::from(-1);
        num_answer: ValueNumber::from(42);
        num_minus_answer: ValueNumber::from(-42);
        string_ident: ValueString::from("ident".to_owned().into_boxed_str());
        string_spaced: ValueString::from("this string has space".to_owned().into_boxed_str());
        string_escaped: ValueString::from("this\tstring\nuses\x42escapes\u{3213}".to_owned().into_boxed_str());
        string_strange: ValueString::from("😃💁 People
        •🐻🌻 Animals
        •🍔🍹 Food
        •🎷⚽️ Activities
        •🚘🌇 Travel
        •💡🎉 Objects
        •💖🔣 Symbols
        •🎌🏳️‍🌈 Flags".to_owned().into_boxed_str());
        list_empty: ValueList::from_iter([]);
        list_omogeneus: ValueList::from_iter([1,2,3,4,-3,-5].map(|v| Value::Number(v.into())));
        list_eterogeneus: ValueList::from_iter([
            ValueNull.into(),
            ValueString::from("hey".to_owned().into_boxed_str()).into(),
            ValueBool::FALSE.into(),
            ValueNumber::from(76).into(),
        ]);
        list_nested: ValueList::from_iter([
            ValueNull.into(),
            ValueString::from("hey".to_owned().into_boxed_str()).into(),
            ValueList::from_iter([
                ValueNull.into(),
                ValueString::from("hey".to_owned().into_boxed_str()).into(),
                ValueBool::FALSE.into(),
                ValueNumber::from(76).into(),
            ]).into(),
            ValueBool::FALSE.into(),
            ValueNumber::from(76).into(),
        ]);
        map_empty: ValueMap::from_iter([]);
        map_with_ident_keys: ValueMap::from_iter([
            ("all".to_owned().into_boxed_str().into(),ValueNull.into()),
            ("b0".to_owned().into_boxed_str().into(),ValueString::from("hey".to_owned().into_boxed_str()).into()),
            ("_c".to_owned().into_boxed_str().into(),ValueBool::FALSE.into()),
        ]);
        map_with_key_not_idents: ValueMap::from_iter([
            ("d".to_owned().into_boxed_str().into(),ValueNull.into()),
            ("hello baby".to_owned().into_boxed_str().into(),ValueString::from("hey".to_owned().into_boxed_str()).into()),
            ("\0".to_owned().into_boxed_str().into(),ValueBool::FALSE.into()),
        ]);

        complex_nested: ValueMap::from_iter([
            ("d".to_owned().into_boxed_str().into(),ValueMap::from_iter([
                ("all".to_owned().into_boxed_str().into(),ValueNull.into()),
                ("b0".to_owned().into_boxed_str().into(),ValueString::from("hey".to_owned().into_boxed_str()).into()),
                ("_c".to_owned().into_boxed_str().into(),ValueBool::FALSE.into()),
            ]).into()),
            ("hello baby".to_owned().into_boxed_str().into(),ValueList::from_iter([
                ValueNull.into(),
                ValueString::from("hey".to_owned().into_boxed_str()).into(),
                ValueList::from_iter([
                    ValueNull.into(),
                    ValueString::from("hey".to_owned().into_boxed_str()).into(),
                    ValueBool::FALSE.into(),
                    ValueNumber::from(-76).into(),
                ]).into(),
                ValueMap::from_iter([
                    ("d".to_owned().into_boxed_str().into(),ValueMap::from_iter([
                        ("all".to_owned().into_boxed_str().into(),ValueNull.into()),
                        ("b0".to_owned().into_boxed_str().into(),ValueString::from("hey".to_owned().into_boxed_str()).into()),
                        ("_c".to_owned().into_boxed_str().into(),ValueBool::FALSE.into()),
                    ]).into()),
                    ("hello baby".to_owned().into_boxed_str().into(),ValueList::from_iter([
                        ValueNull.into(),
                        ValueString::from("hey".to_owned().into_boxed_str()).into(),
                        ValueList::from_iter([
                            ValueNull.into(),
                            ValueString::from("hey".to_owned().into_boxed_str()).into(),
                            ValueBool::FALSE.into(),
                            ValueNumber::from(76).into(),
                        ]).into(),
                        ValueBool::FALSE.into(),
                        ValueNumber::from(76).into(),
                    ]).into()),
                    ("\0".to_owned().into_boxed_str().into(),ValueBool::FALSE.into()),
                ]).into(),
                ValueNumber::from(76).into(),
            ]).into()),
            ("\0".to_owned().into_boxed_str().into(),ValueBool::FALSE.into()),
        ]);
    }
}
