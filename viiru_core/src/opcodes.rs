use std::{collections::HashMap, sync::LazyLock};

use crate::{
    runtime::Runtime,
    spec::{spec, DropdownOption, Spec},
};

#[rustfmt::skip] // screw you you're not ruining my thick fat alignment
pub static BLOCKS: LazyLock<HashMap<String, Spec>> = LazyLock::new(|| {
    HashMap::from_iter([
        // this DSL has become complex and arcane over time. I need to store these in a proper config file
        ("motion_movesteps".into(),              spec("{5F95F8/FFFFFF/4472C6}move (STEPS=10.0) steps")),
        ("motion_turnright".into(),              spec("{5F95F8/FFFFFF/4472C6}turn [$CLOCKWISE] (DEGREES=15.0) degrees")),
        ("motion_turnleft".into(),               spec("{5F95F8/FFFFFF/4472C6}turn [$ANTICLOCKWISE] (DEGREES=15.0) degrees")),
        ("motion_goto".into(),                   spec("{5F95F8/FFFFFF/4472C6}go to (TO=motion_goto_menu)")),
        ("motion_gotoxy".into(),                 spec("{5F95F8/FFFFFF/4472C6}go to x: (X=0.0) y: (Y=0.0)")),
        ("motion_glideto".into(),                spec("{5F95F8/FFFFFF/4472C6}glide (SECS=1.0) secs to (TO=motion_glideto_menu)")),
        ("motion_glidesecstoxy".into(),          spec("{5F95F8/FFFFFF/4472C6}glide (SECS=1.0) secs to x: (X=0.0) y: (Y=0.0)")),
        ("motion_pointindirection".into(),       spec("{5F95F8/FFFFFF/4472C6}point in direction (DIRECTION=90.0)")),
        ("motion_pointtowards".into(),           spec("{5F95F8/FFFFFF/4472C6}point towards (TOWARDS=motion_pointtowards_menu)")),
        ("motion_changexby".into(),              spec("{5F95F8/FFFFFF/4472C6}change x by (DX=10.0)")),
        ("motion_setx".into(),                   spec("{5F95F8/FFFFFF/4472C6}set x to (X=0.0)")),
        ("motion_changeyby".into(),              spec("{5F95F8/FFFFFF/4472C6}change y by (DY=10.0)")),
        ("motion_sety".into(),                   spec("{5F95F8/FFFFFF/4472C6}set y to (Y=0.0)")),
        ("motion_ifonedgebounce".into(),         spec("{5F95F8/FFFFFF/4472C6}if on edge, bounce")),
        ("motion_setrotationstyle".into(),       spec("{5F95F8/FFFFFF/4472C6}set rotation style [STYLE&`left-right`&`don't rotate`&`all around`]")),
        ("motion_xposition".into(),              spec("(5F95F8/FFFFFF/4472C6)x position")),
        ("motion_yposition".into(),              spec("(5F95F8/FFFFFF/4472C6)y position")),
        ("motion_direction".into(),              spec("(5F95F8/FFFFFF/4472C6)direction")),
        ("looks_sayforsecs".into(),              spec("{9268F7/FFFFFF/714FC4}say (MESSAGE=`Hello!`) for (SECS=2.0) seconds")),
        ("looks_say".into(),                     spec("{9268F7/FFFFFF/714FC4}say (MESSAGE=`Hello!`)")),
        ("looks_thinkforsecs".into(),            spec("{9268F7/FFFFFF/714FC4}think (MESSAGE=`Hmm...`) for (SECS=2.0) seconds")),
        ("looks_think".into(),                   spec("{9268F7/FFFFFF/714FC4}think (MESSAGE=`Hmm...`)")),
        ("looks_switchcostumeto".into(),         spec("{9268F7/FFFFFF/714FC4}switch costume to (COSTUME=looks_costume)")),
        ("looks_nextcostume".into(),             spec("{9268F7/FFFFFF/714FC4}next costume")),
        ("looks_switchbackdropto".into(),        spec("{9268F7/FFFFFF/714FC4}switch backdrop to (BACKDROP=looks_backdrops)")),
        ("looks_switchbackdroptoandwait".into(), spec("{9268F7/FFFFFF/714FC4}switch backdrop to (BACKDROP=looks_backdrops) and wait")),
        ("looks_nextbackdrop".into(),            spec("{9268F7/FFFFFF/714FC4}next backdrop")),
        ("looks_changesizeby".into(),            spec("{9268F7/FFFFFF/714FC4}change size by (CHANGE=10.0)")),
        ("looks_setsizeto".into(),               spec("{9268F7/FFFFFF/714FC4}set size to (SIZE=100.0)%")),
        ("looks_changeeffectby".into(),          spec("{9268F7/FFFFFF/714FC4}change [EFFECT/COLOR=`color`/FISHEYE=`fisheye`/WHIRL=`whirl`/PIXELATE=`pixelate`/MOSAIC=`mosaic`/BRIGHTNESS=`brightness`/GHOST=`ghost`] effect by (CHANGE=25.0)")),
        ("looks_seteffectto".into(),             spec("{9268F7/FFFFFF/714FC4}set [EFFECT/COLOR=`color`/FISHEYE=`fisheye`/WHIRL=`whirl`/PIXELATE=`pixelate`/MOSAIC=`mosaic`/BRIGHTNESS=`brightness`/GHOST=`ghost`] effect to (VALUE=0.0)")),
        ("looks_cleargraphiceffects".into(),     spec("{9268F7/FFFFFF/714FC4}clear graphic effects")),
        ("looks_show".into(),                    spec("{9268F7/FFFFFF/714FC4}show")),
        ("looks_hide".into(),                    spec("{9268F7/FFFFFF/714FC4}hide")),
        ("looks_gotofrontback".into(),           spec("{9268F7/FFFFFF/714FC4}go to [FRONT_BACK&`front`&`back`] layer")),
        ("looks_goforwardbackwardlayers".into(), spec("{9268F7/FFFFFF/714FC4}go [FORWARD_BACKWARD&`forward`&`backward`] (NUM=1.0) layers")),
        ("looks_costumenumbername".into(),       spec("(9268F7/FFFFFF/714FC4)costume [NUMBER_NAME&`number`&`name`]")),
        ("looks_backdropnumbername".into(),      spec("(9268F7/FFFFFF/714FC4)backdrop [NUMBER_NAME&`number`&`name`]")),
        ("looks_size".into(),                    spec("(9268F7/FFFFFF/714FC4)size")),
        ("sound_playuntildone".into(),           spec("{C169C9/FFFFFF/AF4BB7}play sound (SOUND_MENU=sound_sounds_menu) until done")),
        ("sound_play".into(),                    spec("{C169C9/FFFFFF/AF4BB7}start sound (SOUND_MENU=sound_sounds_menu)")),
        ("sound_stopallsounds".into(),           spec("{C169C9/FFFFFF/AF4BB7}stop all sounds")),
        ("sound_changeeffectby".into(),          spec("{C169C9/FFFFFF/AF4BB7}change [EFFECT/PITCH=`pitch`/PAN=`pan left/right`] effect by (VALUE=10.0)")),
        ("sound_seteffectto".into(),             spec("{C169C9/FFFFFF/AF4BB7}set [EFFECT/PITCH=`pitch`/PAN=`pan left/right`] effect to (VALUE=100.0)")),
        ("sound_cleareffects".into(),            spec("{C169C9/FFFFFF/AF4BB7}clear sound effects")),
        ("sound_changevolumeby".into(),          spec("{C169C9/FFFFFF/AF4BB7}change volume by (VOLUME=-10.0)")),
        ("sound_setvolumeto".into(),             spec("{C169C9/FFFFFF/AF4BB7}set volume to (VOLUME=100.0)%")),
        ("sound_volume".into(),                  spec("(C169C9/FFFFFF/AF4BB7)volume")),
        ("event_whenflagclicked".into(),         spec("{F5C242/FFFFFF/C49B33^when [$FLAG] clicked")),
        ("event_whenkeypressed".into(),          spec("{F5C242/FFFFFF/C49B33^when [KEY_OPTION&`space`&`up arrow`&`down arrow`&`right arrow`&`left arrow`&`any`&`a`&`b`&`c`&`d`&`e`&`f`&`g`&`h`&`i`&`j`&`k`&`l`&`m`&`n`&`o`&`p`&`q`&`r`&`s`&`t`&`u`&`v`&`w`&`x`&`y`&`z`&`0`&`1`&`2`&`3`&`4`&`5`&`6`&`7`&`8`&`9`] key pressed")), // case in point wtf is this
        ("event_whenthisspriteclicked".into(),   spec("{F5C242/FFFFFF/C49B33^when this sprite clicked")),
        ("event_whenstageclicked".into(),        spec("{F5C242/FFFFFF/C49B33^when stage clicked")),
        ("event_whenbackdropswitchesto".into(),  spec("{F5C242/FFFFFF/C49B33^when backdrop switches to [BACKDROP]")),
        ("event_whengreaterthan".into(),         spec("{F5C242/FFFFFF/C49B33^when [WHENGREATERTHANMENU/LOUDNESS=`loudness`] > (VALUE=10.0)")),
        ("event_whenbroadcastreceived".into(),   spec("{F5C242/FFFFFF/C49B33^when I receive [BROADCAST_OPTION]")),
        ("event_broadcast".into(),               spec("{F5C242/FFFFFF/C49B33}broadcast (BROADCAST_INPUT=event_broadcast_menu)")),
        ("event_broadcastandwait".into(),        spec("{F5C242/FFFFFF/C49B33}broadcast (BROADCAST_INPUT=event_broadcast_menu) and wait")),
        ("control_wait".into(),                  spec("{F3AF43/FFFFFF/C58E36}wait (DURATION=1.0) seconds")),
        ("control_repeat".into(),                spec("{F3AF43/FFFFFF/C58E36}[:SUBSTACK]repeat (TIMES=10.0)\n{SUBSTACK}\n\t")),
        ("control_forever".into(),               spec("{F3AF43/FFFFFF/C58E36}[:SUBSTACK]forever\n{SUBSTACK}\n\t")),
        ("control_if".into(),                    spec("{F3AF43/FFFFFF/C58E36}[:SUBSTACK]if <CONDITION> then\n{SUBSTACK}\n\t")),
        ("control_if_else".into(),               spec("{F3AF43/FFFFFF/C58E36}[:SUBSTACK]if <CONDITION> then\n{SUBSTACK}\n[:SUBSTACK2]else\n{SUBSTACK2}\n\t")),
        ("control_wait_until".into(),            spec("{F3AF43/FFFFFF/C58E36}wait until <CONDITION>")),
        ("control_repeat_until".into(),          spec("{F3AF43/FFFFFF/C58E36}[:SUBSTACK]repeat until <CONDITION>\n{SUBSTACK}\n\t")),
        ("control_stop".into(),                  spec("{F3AF43/FFFFFF/C58E36}stop [STOP_OPTION]")),
        ("control_start_as_clone".into(),        spec("{F3AF43/FFFFFF/C58E36^when I start as a clone")),
        ("control_create_clone_of".into(),       spec("{F3AF43/FFFFFF/C58E36}create a clone of (CLONE_OPTION=control_create_clone_of_menu)")),
        ("control_delete_this_clone".into(),     spec("{F3AF43/FFFFFF/C58E36}delete this clone")),
        ("sensing_touchingobject".into(),        spec("<71AFD2/FFFFFF/4B8CB4>touching (TOUCHINGOBJECTMENU=sensing_touchingobjectmenu)?")),
        ("sensing_touchingcolor".into(),         spec("<71AFD2/FFFFFF/4B8CB4>touching color (COLOR=#3D7088)?")),
        ("sensing_coloristouchingcolor".into(),  spec("<71AFD2/FFFFFF/4B8CB4>color (COLOR=#461C5B) is touching (COLOR2=#5BB033)?")),
        ("sensing_distanceto".into(),            spec("(71AFD2/FFFFFF/4B8CB4)distance to(DISTANCETOMENU=sensing_distancetomenu)")),
        ("sensing_askandwait".into(),            spec("{71AFD2/FFFFFF/4B8CB4}ask (QUESTION=`What's your name?`) and wait")),
        ("sensing_answer".into(),                spec("(71AFD2/FFFFFF/4B8CB4)answer")),
        ("sensing_keypressed".into(),            spec("<71AFD2/FFFFFF/4B8CB4>key (KEY_OPTION=sensing_keyoptions) pressed?")),
        ("sensing_mousedown".into(),             spec("<71AFD2/FFFFFF/4B8CB4>mouse down?")),
        ("sensing_mousex".into(),                spec("(71AFD2/FFFFFF/4B8CB4)mouse x")),
        ("sensing_mousey".into(),                spec("(71AFD2/FFFFFF/4B8CB4)mouse y")),
        ("sensing_setdragmode".into(),           spec("{71AFD2/FFFFFF/4B8CB4}set drag mode [DRAG_MODE&`draggable`&`not draggable`]")),
        ("sensing_loudness".into(),              spec("(71AFD2/FFFFFF/4B8CB4)loudness")),
        ("sensing_timer".into(),                 spec("(71AFD2/FFFFFF/4B8CB4)timer")),
        ("sensing_resettimer".into(),            spec("{71AFD2/FFFFFF/4B8CB4}reset timer")),
        ("sensing_of".into(),                    spec("(71AFD2/FFFFFF/4B8CB4)[PROPERTY] of (OBJECT=sensing_of_object_menu)")),
        ("sensing_current".into(),               spec("(71AFD2/FFFFFF/4B8CB4)current [CURRENTMENU/YEAR=`year`/MONTH=`month`/DATE=`date`/DAYOFWEEK=`day of week`/HOUR=`hour`/MINUTE=`minute`/SECOND=`second`]")),
        ("sensing_dayssince2000".into(),         spec("(71AFD2/FFFFFF/4B8CB4)days since 2000")),
        ("sensing_username".into(),              spec("(71AFD2/FFFFFF/4B8CB4)username")),
        ("operator_add".into(),                  spec("(74BE65/FFFFFF/529244)(NUM1=@) + (NUM2=@)")),
        ("operator_subtract".into(),             spec("(74BE65/FFFFFF/529244)(NUM1=@) - (NUM2=@)")),
        ("operator_multiply".into(),             spec("(74BE65/FFFFFF/529244)(NUM1=@) * (NUM2=@)")),
        ("operator_divide".into(),               spec("(74BE65/FFFFFF/529244)(NUM1=@) / (NUM2=@)")),
        ("operator_random".into(),               spec("(74BE65/FFFFFF/529244)pick random (FROM=1.0) to (TO=10.0)")),
        ("operator_gt".into(),                   spec("<74BE65/FFFFFF/529244>(OPERAND1=@) > (OPERAND2=50.0)")),
        ("operator_lt".into(),                   spec("<74BE65/FFFFFF/529244>(OPERAND1=@) [<] (OPERAND2=50.0)")),
        ("operator_equals".into(),               spec("<74BE65/FFFFFF/529244>(OPERAND1=@) = (OPERAND2=50.0)")),
        ("operator_and".into(),                  spec("<74BE65/FFFFFF/529244><OPERAND1> and <OPERAND2>")),
        ("operator_or".into(),                   spec("<74BE65/FFFFFF/529244><OPERAND1> or <OPERAND2>")),
        ("operator_not".into(),                  spec("<74BE65/FFFFFF/529244>not <OPERAND>")),
        ("operator_join".into(),                 spec("(74BE65/FFFFFF/529244)join (STRING1=`apple`) (STRING2=`banana`)")),
        ("operator_letter_of".into(),            spec("(74BE65/FFFFFF/529244)letter (LETTER=1.0) of (STRING=`apple`)")),
        ("operator_length".into(),               spec("(74BE65/FFFFFF/529244)length of (STRING=`apple`)")),
        ("operator_contains".into(),             spec("{74BE65/FFFFFF/529244}(STRING1=`apple`) contains (STRING2=`a`)?")),
        ("operator_mod".into(),                  spec("(74BE65/FFFFFF/529244)(NUM1=@) mod (NUM2=@)")),
        ("operator_round".into(),                spec("(74BE65/FFFFFF/529244)round (NUM=@)")),
        ("operator_mathop".into(),               spec("(74BE65/FFFFFF/529244)[OPERATOR&`abs`&`floor`&`ceiling`&`sqrt`&`sin`&`cos`&`tan`&`asin`&`acos`&`atan`&`ln`&`log`&`e ^`&`10 ^`] of (NUM=@)")),
        ("data_variable".into(),                 spec("(F0923C/FFFFFF/FFFFFF)[&VARIABLE]")), // dynamic label (field contains ID)
        ("data_setvariableto".into(),            spec("{F0923C/FFFFFF/CD742A}set [VARIABLE] to (VALUE=0.0)")),
        ("data_changevariableby".into(),         spec("{F0923C/FFFFFF/CD742A}change [VARIABLE] by (VALUE=1.0)")),
        ("data_showvariable".into(),             spec("{F0923C/FFFFFF/CD742A}show variable [VARIABLE]")),
        ("data_hidevariable".into(),             spec("{F0923C/FFFFFF/CD742A}hide variable [VARIABLE]")),
        ("data_listcontents".into(),             spec("(ED7035/FFFFFF/D55825)[&LIST]")), // dynamic label (field contains ID)
        ("data_addtolist".into(),                spec("{ED7035/FFFFFF/D55825}add (ITEM=`thing`) to [LIST]")),
        ("data_deleteoflist".into(),             spec("{ED7035/FFFFFF/D55825}delete (INDEX=1.0) of [LIST]")),
        ("data_deletealloflist".into(),          spec("{ED7035/FFFFFF/D55825}delete all of [LIST]")),
        ("data_insertatlist".into(),             spec("{ED7035/FFFFFF/D55825}insert (ITEM=`thing`) at (INDEX=1.0) of [LIST]")),
        ("data_replaceitemoflist".into(),        spec("{ED7035/FFFFFF/D55825}replace item (INDEX=1.0) of [LIST] with (ITEM=`thing`)")),
        ("data_itemoflist".into(),               spec("(ED7035/FFFFFF/D55825)item (INDEX=1.0) of [LIST]")),
        ("data_itemnumoflist".into(),            spec("(ED7035/FFFFFF/D55825)item # of (ITEM=`thing`) in [LIST]")),
        ("data_lengthoflist".into(),             spec("(ED7035/FFFFFF/D55825)length of [LIST]")),
        ("data_listcontainsitem".into(),         spec("<ED7035/FFFFFF/D55825>[LIST] contains (ITEM=`thing`)?")),
        ("data_showlist".into(),                 spec("{ED7035/FFFFFF/D55825}show list [LIST]")),
        ("data_hidelist".into(),                 spec("{ED7035/FFFFFF/D55825}hide list [LIST]")),
        // internal dropdowns
        ("motion_goto_menu".into(),              spec("(517FD1/FFFFFF/4472C6![TO]")),
        ("motion_glideto_menu".into(),           spec("(517FD1/FFFFFF/4472C6![TO]")),
        ("motion_pointtowards_menu".into(),      spec("(517FD1/FFFFFF/4472C6![TOWARDS]")),
        ("looks_costume".into(),                 spec("(7F5ECF/FFFFFF/714FC4![COSTUME]")),
        ("looks_backdrops".into(),               spec("(7F5ECF/FFFFFF/714FC4![BACKDROP]")),
        ("sound_sounds_menu".into(),             spec("(BB57C3/FFFFFF/AF4BB7![SOUND_MENU]")),
        ("event_broadcast_menu".into(),          spec("(DDAE3B/FFFFFF/C49B33![BROADCAST_OPTION]")),
        ("control_create_clone_of_menu".into(),  spec("(E09F3B/FFFFFF/C58E36![CLONE_OPTION]")),
        ("sensing_touchingobjectmenu".into(),    spec("(62A6CD/FFFFFF/4B8CB4![TOUCHINGOBJECTMENU]")),
        ("sensing_distancetomenu".into(),        spec("(62A6CD/FFFFFF/4B8CB4![DISTANCETOMENU]")),
        ("sensing_keyoptions".into(),            spec("(62A6CD/FFFFFF/4B8CB4![KEY_OPTION&`space`&`up arrow`&`down arrow`&`right arrow`&`left arrow`&`any`&`a`&`b`&`c`&`d`&`e`&`f`&`g`&`h`&`i`&`j`&`k`&`l`&`m`&`n`&`o`&`p`&`q`&`r`&`s`&`t`&`u`&`v`&`w`&`x`&`y`&`z`&`0`&`1`&`2`&`3`&`4`&`5`&`6`&`7`&`8`&`9`]")),
        ("sensing_of_object_menu".into(),        spec("(62A6CD/FFFFFF/4B8CB4![OBJECT]")),
        // shadow blocks
        ("math_number".into(),                   spec("(FFFFFF/595E73/FFFFFF![*NUM]")), // dynamic label (field contains text)
        ("math_integer".into(),                  spec("(FFFFFF/595E73/FFFFFF![*NUM]")), // dynamic label (field contains text)
        ("math_whole_number".into(),             spec("(FFFFFF/595E73/FFFFFF![*NUM]")), // dynamic label (field contains text)
        ("math_positive_number".into(),          spec("(FFFFFF/595E73/FFFFFF![*NUM]")), // dynamic label (field contains text)
        ("math_angle".into(),                    spec("(FFFFFF/595E73/FFFFFF![*NUM]")), // dynamic label (field contains text)
        ("text".into(),                          spec("(FFFFFF/595E73/FFFFFF![*TEXT]")), // dynamic label (field contains text)
        ("colour_picker".into(),                 spec("(FFFFFF/595E73/FFFFFF![#COLOUR]")), // dynamic label (field contains color)
    ])
});

fn dup(s: &str) -> DropdownOption {
    DropdownOption {
        value: s.to_string(),
        display: s.to_string(),
    }
}

fn yup(value: &str, display: &str) -> DropdownOption {
    DropdownOption {
        value: value.to_string(),
        display: display.to_string(),
    }
}

pub fn dropdown_options(runtime: &Runtime, block_id: &str, opcode: &str) -> Vec<DropdownOption> {
    if let Some(opts) = &BLOCKS[opcode].static_dropdown_options {
        opts.clone()
    } else {
        match opcode {
            "control_stop" => {
                if runtime.blocks[block_id].next_id.is_some() {
                    vec![dup("other scripts in sprite")]
                } else {
                    vec![
                        dup("all"),
                        dup("this script"),
                        dup("other scripts in sprite"),
                    ]
                }
            }
            "control_create_clone_of_menu" => {
                // todo: targets
                todo!("targets")
                // yup("_myself_", "myself")
            }
            "data_setvariableto"
            | "data_changevariableby"
            | "data_showvariable"
            | "data_hidevariable" => {
                todo!("variable dropdowns")
            }
            "data_addtolist"
            | "data_deleteoflist"
            | "data_deletealloflist"
            | "data_insertatlist"
            | "data_replaceitemoflist"
            | "data_itemoflist"
            | "data_itemnumoflist"
            | "data_lengthoflist"
            | "data_listcontainsitem"
            | "data_showlist"
            | "data_hidelist" => {
                todo!("list dropdowns")
            }
            "event_whenbackdropswitchesto" | "looks_backdrops" => {
                todo!("backdrop dropdowns")
            }
            "looks_costume" => {
                todo!("costume dropdowns")
            }
            "motion_pointtowards_menu" => {
                todo!("targets")
                // yup("_mouse_", "mouse-pointer")
                // yup("_random_", "random direction")
            }
            "motion_goto_menu" | "motion_glideto_menu" => {
                todo!("targets")
                // yup("_mouse_", "mouse-pointer")
                // yup("_random_", "random position")
            }
            "sensing_touchingobjectmenu" => {
                todo!("targets")
                // yup("_mouse_", "mouse-pointer")
                // yup("_edge_", "edge")
            }
            "sensing_distancetomenu" => {
                todo!("targets")
                // yup("_mouse_", "mouse-pointer")
            }
            "sensing_of_object_menu" => {
                todo!("targets")
                // yup("_stage_", "Stage")
            }
            "sensing_of" => {
                todo!("target-dependant properties")
                // x position
                // y position
                // direction
                // costume #
                // costume name
                // size
                // volume
                // backdrop #
                // backdrop name
            }
            "sound_sounds_menu" => {
                todo!("sounds")
            }
            _ => unimplemented!(),
        }
    }
}

pub const NUMBERS_ISH: [&str; 5] = [
    "math_number",
    "math_integer",
    "math_whole_number",
    "math_positive_number",
    "math_angle",
];

pub const TOOLBOX: &[&str] = &[
    "motion_movesteps",
    "motion_turnright",
    "motion_turnleft",
    "motion_goto",
    "motion_gotoxy",
    "motion_glideto",
    "motion_glidesecstoxy",
    "motion_pointindirection",
    "motion_pointtowards",
    "motion_changexby",
    "motion_setx",
    "motion_changeyby",
    "motion_sety",
    "motion_ifonedgebounce",
    "motion_setrotationstyle",
    "motion_xposition",
    "motion_yposition",
    "motion_direction",
    "looks_sayforsecs",
    "looks_say",
    "looks_thinkforsecs",
    "looks_think",
    "looks_switchcostumeto",
    "looks_nextcostume",
    "looks_switchbackdropto",
    "looks_switchbackdroptoandwait",
    "looks_nextbackdrop",
    "looks_changesizeby",
    "looks_setsizeto",
    "looks_changeeffectby",
    "looks_seteffectto",
    "looks_cleargraphiceffects",
    "looks_show",
    "looks_hide",
    "looks_gotofrontback",
    "looks_goforwardbackwardlayers",
    "looks_costumenumbername",
    "looks_backdropnumbername",
    "looks_size",
    "sound_playuntildone",
    "sound_play",
    "sound_stopallsounds",
    "sound_changeeffectby",
    "sound_seteffectto",
    "sound_cleareffects",
    "sound_changevolumeby",
    "sound_setvolumeto",
    "sound_volume",
    "event_whenflagclicked",
    "event_whenkeypressed",
    "event_whenthisspriteclicked",
    "event_whenstageclicked",
    "event_whenbackdropswitchesto",
    "event_whengreaterthan",
    "event_whenbroadcastreceived",
    "event_broadcast",
    "event_broadcastandwait",
    "control_wait",
    "control_repeat",
    "control_forever",
    "control_if",
    "control_if_else",
    "control_wait_until",
    "control_repeat_until",
    "control_stop",
    "control_start_as_clone",
    "control_create_clone_of",
    "control_delete_this_clone",
    "sensing_touchingobject",
    "sensing_touchingcolor",
    "sensing_coloristouchingcolor",
    "sensing_distanceto",
    "sensing_askandwait",
    "sensing_answer",
    "sensing_keypressed",
    "sensing_mousedown",
    "sensing_mousex",
    "sensing_mousey",
    "sensing_setdragmode",
    "sensing_loudness",
    "sensing_timer",
    "sensing_resettimer",
    "sensing_of",
    "sensing_current",
    "sensing_dayssince2000",
    "sensing_username",
    "operator_add",
    "operator_subtract",
    "operator_multiply",
    "operator_divide",
    "operator_random",
    "operator_gt",
    "operator_lt",
    "operator_equals",
    "operator_and",
    "operator_or",
    "operator_not",
    "operator_join",
    "operator_letter_of",
    "operator_length",
    "operator_contains",
    "operator_mod",
    "operator_round",
    "operator_mathop",
    // "data_variable", // extra fields
    "data_setvariableto",
    "data_changevariableby",
    "data_showvariable",
    "data_hidevariable",
    // "data_listcontents", // extra fields
    "data_addtolist",
    "data_deleteoflist",
    "data_deletealloflist",
    "data_insertatlist",
    "data_replaceitemoflist",
    "data_itemoflist",
    "data_itemnumoflist",
    "data_lengthoflist",
    "data_listcontainsitem",
    "data_showlist",
    "data_hidelist",
    // "procedures_definition", // extra fields
    // "procedures_call", // extra fields
    // "procedures_prototype", // extra fields
    // "argument_reporter_string_number", // extra fields
    // "argument_reporter_boolean", // extra fields
];
