use std::{collections::HashMap, sync::LazyLock};

use crate::spec::{spec, Spec};

pub static BLOCKS: LazyLock<HashMap<&'static str, Spec>> = LazyLock::new(|| {
    HashMap::from_iter([
        ("motion_movesteps",              spec("move(STEPS=10.0)steps")),// STEPS
        ("motion_turnright",              spec("turn [$CLOCKWISE](DEGREES=15.0)degrees")),// DEGREES
        ("motion_turnleft",               spec("turn [$ANTICLOCKWISE](DEGREES=15.0)degrees")),// DEGREES
        ("motion_goto",                   spec("go to(TO=motion_goto_menu)")),// TO
        ("motion_gotoxy",                 spec("go to x:(X=0.0)y:(Y=0.0)")),// X Y
        ("motion_glideto",                spec("glide(SECS=1.0)secs to(TO=motion_glideto_menu)")),// SECS TO
        ("motion_glidesecstoxy",          spec("glide(SECS=1.0)secs to x:(X=0.0)y:(Y=0.0)")),// SECS X Y
        ("motion_pointindirection",       spec("point in direction(DIRECTION=90.0)")),// DIRECTION
        ("motion_pointtowards",           spec("point towards(TOWARDS=motion_pointtowards_menu)")),// TOWARDS
        ("motion_changexby",              spec("change x by(DX=10.0)")),// DX
        ("motion_setx",                   spec("set x to(X=0.0)")),// X
        ("motion_changeyby",              spec("change y by(DY=10.0)")),// DY
        ("motion_sety",                   spec("set y to(Y=0.0)")),// Y
        ("motion_ifonedgebounce",         spec("if on edge, bounce")),//
        ("motion_setrotationstyle",       spec("set rotation style[STYLE]")),//
        ("motion_xposition",              spec("x position")),//
        ("motion_yposition",              spec("y position")),// 
        ("motion_direction",              spec("direction")),//
        ("looks_sayforsecs",              spec("say(MESSAGE=\"Hello!\")for(SECS=2.0)seconds")),// MESSAGE SECS
        ("looks_say",                     spec("say(MESSAGE=\"Hello!\")")),// MESSAGE
        ("looks_thinkforsecs",            spec("think(MESSAGE=\"Hmm...\")for(SECS=2.0)secondss")),// MESSAGE SECS
        ("looks_think",                   spec("think(MESSAGE=\"Hmm...\")")),// MESSAGE
        ("looks_switchcostumeto",         spec("switch costume to(COSTUME=looks_costume)")),// COSTUME
        ("looks_nextcostume",             spec("next costume")),//
        ("looks_switchbackdropto",        spec("switch backdrop to(BACKDROP=looks_backdrops)")),// BACKDROP
        ("looks_switchbackdroptoandwait", spec("swith backdrop to(BACKDROP=looks_backdrops)and wait")),// BACKDROP
        ("looks_nextbackdrop",            spec("next backdrop")),//
        ("looks_changesizeby",            spec("change size by(CHANGE=10.0)")),// CHANGE
        ("looks_setsizeto",               spec("set size to(SIZE=100.0)%")),// SIZE
        ("looks_changeeffectby",          spec("change[EFFECT]effect by(CHANGE=25.0)")),// CHANGE
        ("looks_seteffectto",             spec("set[EFFECT]effect to(VALUE=0.0)")),// VALUE
        ("looks_cleargraphiceffects",     spec("clear graphic effects")),//
        ("looks_show",                    spec("show")),//
        ("looks_hide",                    spec("hide")),//
        ("looks_gotofrontback",           spec("go to[FRONT_BACK]layer")),//
        ("looks_goforwardbackwardlayers", spec("go[FORWARD_BACKWARD](NUM=1.0)layers")),// NUM
        ("looks_costumenumbername",       spec("costume[NUMBER_NAME]")),//
        ("looks_backdropnumbername",      spec("backdrop[NUMBER_NAME]")),//
        ("looks_size",                    spec("size")),//
        ("sound_playuntildone",           spec("play sound(SOUND_MENU=sound_sounds_menu)until done")),// SOUND_MENU
        ("sound_play",                    spec("start sound(SOUND_MENU=sound_sounds_menu)")),// SOUND_MENU
        ("sound_stopallsounds",           spec("stop all sounds")),//
        ("sound_changeeffectby",          spec("change[EFFECT]effect by(VALUE=10.0)")),// VALUE
        ("sound_seteffectto",             spec("set[EFFECT]effect to(VALUE=100.0)")),// VALUE
        ("sound_cleareffects",            spec("clear sound effects")),//
        ("sound_changevolumeby",          spec("change volume by(VOLUME=-10.0)")),// VOLUME
        ("sound_setvolumeto",             spec("set volume to(VOLUME=100.0)%")),// VOLUME
        ("sound_volume",                  spec("volume")),//
        ("event_whenflagclicked",         spec("when [$FLAG] clicked")),//
        ("event_whenkeypressed",          spec("when[KEY_OPTION]key pressed")),//
        ("event_whenthisspriteclicked",   spec("when this sprite clicked")),//
        // ("event_whenstageclicked",        spec("when stage clicked")),//
        ("event_whenbackdropswitchesto",  spec("when backdrop switches to[BACKDROP]")),//
        ("event_whengreaterthan",         spec("when[WHENGREATERTHANMENU]>(VALUE=10.0)")),// VALUE
        ("event_whenbroadcastreceived",   spec("when I receive[BROADCAST_OPTION]")),//
        ("event_broadcast",               spec("broadcast(BROADCAST_INPUT=event_broadcast_menu)")),// BROADCAST_INPUT
        ("event_broadcastandwait",        spec("broadcast(BROADCAST_INPUT=event_broadcast_menu)and wait")),// BROADCAST_INPUT
        ("control_wait",                  spec("wait(DURATION=1.0)seconds")),// DURATION
        ("control_repeat",                spec("repeat(TIMES)\n{SUBSTACK}\n\t")),// TIMES SUBSTACK
        ("control_forever",               spec("forever\n{SUBSTACK}\n\t")),// SUBSTACK
        ("control_if",                    spec("if<CONDITION>then\n{SUBSTACK}\n\t")),// CONDITION SUBSTACK
        ("control_if_else",               spec("if<CONDITION>then\n{SUBSTACK}\nelse\n{SUBSTACK2}\n\t")),// CONDITION SUBSTACK SUBSTACK2
        ("control_wait_until",            spec("wait until<CONDITION>")),// CONDITION
        ("control_repeat_until",          spec("repeat until<CONDITION>\n{SUBSTACK}\n\t")),// CONDITION SUBSTACK
        ("control_stop",                  spec("stop[STOP_OPTION]")),//
        ("control_start_as_clone",        spec("when I start as a clone")),//
        ("control_create_clone_of",       spec("create a clone of(CLONE_OPTION=control_create_clone_of_menu)")),// CLONE_OPTION
        ("control_delete_this_clone",     spec("delete this clone")),//
        ("sensing_touchingobject",        spec("touching(TOUCHINGOBJECTMENU=sensing_touchingobjectmenu)?")),// TOUCHINGOBJECTMENU
        ("sensing_touchingcolor",         spec("touching color(COLOR=#3D7088)?")),// COLOR
        ("sensing_coloristouchingcolor",  spec("color(COLOR=#461C5B)is touching(COLOR2=#5BB033)?")),// COLOR COLOR2
        ("sensing_distanceto",            spec("distance to(DISTANCETOMENU=sensing_distancetomenu)")),// DISTANCETOMENU
        ("sensing_askandwait",            spec("ask(QUESTION=\"What's your name?\")and want")),// QUESTION
        ("sensing_answer",                spec("answer")),//
        ("sensing_keypressed",            spec("key(KEY_OPTION=sensing_keyoptions)pressed?")),// KEY_OPTION
        ("sensing_mousedown",             spec("mouse down?")),//
        ("sensing_mousex",                spec("mouse x")),//
        ("sensing_mousey",                spec("mouse y")),//
        ("sensing_setdragmode",           spec("set drag mode[DRAG_MODE]")),//
        ("sensing_loudness",              spec("loudness")),//
        ("sensing_timer",                 spec("timer")),//
        ("sensing_resettimer",            spec("reset timer")),//
        ("sensing_of",                    spec("[PROPERTY]of(OBJECT=sensing_of_object_menu)")),// OBJECT
        ("sensing_current",               spec("current[CURRENTMENU]")),//
        ("sensing_dayssince2000",         spec("days since 2000")),//
        ("sensing_username",              spec("username")),//
        ("operator_add",                  spec("(NUM1=\"\")+(NUM2=\"\")")),// NUM1 NUM2
        ("operator_subtract",             spec("(NUM1=\"\")-(NUM2=\"\")")),// NUM1 NUM2
        ("operator_multiply",             spec("(NUM1=\"\")*(NUM2=\"\")")),// NUM1 NUM2
        ("operator_divide",               spec("(NUM1=\"\")/(NUM2=\"\")")),// NUM1 NUM2
        ("operator_random",               spec("pick random(FROM=1.0)to(TO=10.0)")),// FROM TO
        ("operator_gt",                   spec("(OPERAND1=\"\")>(OPERAND2=50.0)")),// OPERAND1 OPERAND2
        ("operator_lt",                   spec("(OPERAND1=\"\")[<](OPERAND2=50.0)")),// OPERAND1 OPERAND2
        ("operator_equals",               spec("(OPERAND1=\"\")=(OPERAND2=50.0)")),// OPERAND1 OPERAND2
        ("operator_and",                  spec("<OPERAND1>and<OPERAND2>")),// OPERAND1 OPERAND2
        ("operator_or",                   spec("<OPERAND1>or<OPERAND2>")),// OPERAND1 OPERAND2
        ("operator_not",                  spec("not<OPERAND>")),// OPERAND
        ("operator_join",                 spec("join(STRING1=\"apple\")(STRING2=\"banana\")")),// STRING1 STRING2
        ("operator_letter_of",            spec("letter(LETTER=1.0)of(STRING=\"apple\")")),// LETTER STRING
        ("operator_length",               spec("length of(STRING=\"apple\")")),// STRING
        ("operator_contains",             spec("(STRING1=\"apple\")contains(STRING2=\"a\")?")),// STRING1 STRING2
        ("operator_mod",                  spec("(NUM1=\"\")mod(NUM2=\"\")")),// NUM1 NUM2
        ("operator_round",                spec("round(NUM=\"\")")),// NUM
        ("operator_mathop",               spec("[OPERATOR]of(NUM=\"\")")),// NUM
        ("data_setvariableto",            spec("set[VARIABLE]to(VALUE=0.0)")),// VALUE
        ("data_changevariableby",         spec("change[VARIABLE]by(value=1.0)")),// VALUE
        ("data_showvariable",             spec("show variable[VARIABLE]")),//
        ("data_hidevariable",             spec("hide variable[VARIABLE]")),//
        ("data_addtolist",                spec("add(ITEM=\"thing\")to[LIST]")),// ITEM
        ("data_deleteoflist",             spec("delete(INDEX=1.0)of[LIST]")),// INDEX
        ("data_deletealloflist",          spec("delete all of[LIST]")),//
        ("data_insertatlist",             spec("insert(ITEM=\"thing\")at(INDEX=1.0)of[LIST]")),// ITEM INDEX
        ("data_replaceitemoflist",        spec("replace item(INDEX=1.0)of[LIST]with(ITEM=\"thing\")")),// INDEX ITEM
        ("data_itemoflist",               spec("item(INDEX=1.0)of[LIST]")),// INDEX
        ("data_itemnumoflist",            spec("item # of(ITEM=\"thing\")in[LIST]")),// ITEM
        ("data_lengthoflist",             spec("length of[LIST]")),//
        ("data_listcontainsitem",         spec("[LIST]contains(ITEM=\"thing\")?")),// ITEM
        ("data_showlist",                 spec("show list[LIST]")),//
        ("data_hidelist",                 spec("hide list[LIST]")),//
        // internal dropdowns
        ("motion_goto_menu",              spec("[TO]")),
        ("motion_glideto_menu",           spec("[TO]")),
        ("motion_pointtowards_menu",      spec("[TOWARDS]")),
        ("looks_costume",                 spec("[COSTUME]")),
        ("looks_backdrops",               spec("[BACKDROP]")),
        ("sound_sounds_menu",             spec("[SOUND_MENU]")),
        ("event_broadcast_menu",          spec("[BROADCAST_OPTION]")),
        ("control_create_clone_of_menu",  spec("[CLONE_OPTION]")),
        ("sensing_touchingobjectmenu",    spec("[TOUCHINGOBJECTMENU]")),
        ("sensing_distancetomenu",        spec("[DISTANCETOMENU]")),
        ("sensing_keyoptions",            spec("[KEY_OPTION]")),
        ("sensing_of_object_menu",        spec("[OBJECT]")),
    ])
});

pub const OPCODES: &[&str] = &[
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
    "motion_goto_menu",
    "motion_glideto_menu",
    "motion_pointtowards_menu",
    "looks_costume",
    "looks_backdrops",
    "sound_sounds_menu",
    "event_broadcast_menu",
    "control_create_clone_of_menu",
    "sensing_touchingobjectmenu",
    "sensing_distancetomenu",
    "sensing_keyoptions",
    "sensing_of_object_menu",
];

