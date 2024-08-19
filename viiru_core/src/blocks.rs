use std::{collections::HashMap, sync::LazyLock};

use crate::spec::{spec, Spec};

pub static BLOCKS: LazyLock<HashMap<&'static str, Spec>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("control_forever",       spec("forever\n{SUBSTACK}\n")),
        ("control_if",            spec("if<CONDITION>then\n{SUBSTACK}\n")),
        ("event_whenflagclicked", spec("when [#FLAG] clicked")),
        ("looks_sayforsecs",      spec("say(MESSAGE)for(DURATION)seconds")),
        ("operator_equals",       spec("(OPERAND1)=(OPERAND2)")),
    ])
});

// defaults: id -> [id | shadow | nil]
