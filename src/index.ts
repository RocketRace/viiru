// looking at the dependencies, we've got
import fs from 'node:fs';
// utilities
import core from 'viiru_core';
// the custom editor logic
import VM from 'scratch-vm';
// the scratch core VM
import Storage from 'scratch-storage';
// serialization for scratch assets
// eslint-disable-next-line @typescript-eslint/no-require-imports
require('browser-env')(['window', 'document'])
import { Workspace } from 'scratch-blocks';
// ... uhhhhhhhhh so
//
// scratch-blocks contains an UI implementation for the scratch block 
// editor and the VM listens to specifically its events, and it also
// offers convenience methods to interact with the VM's blocks (which
// is exactly what this project is, a block editor)
//
// but those block events are a pain in the ass to construct manually -
// like, writing-template-xml-strings-in-json kind of a pain -
// so I'd much rather pull in the dependency
//
// problem: it's a whole ass UI designed for the browser, and therefore
// uses all sorts of browser-only globals just to initialize
//
// solution: just pull a whole browser environment into nodejs :)
// a huge dependency tree never hurt anybody
const vm = new VM();
const workspace = new Workspace()
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const listener = (vm as any).blockListener
//                           ^ missing from type decl
workspace.addChangeListener(listener)
const storage = new Storage();
vm.attachStorage(storage)

const load = async (path: string) => {
    const buffer = fs.readFileSync(path);
    await vm.loadProject(buffer);
}

const save = async (path: string) => {
    const blob = await vm.saveProjectSb3();
    fs.writeFileSync(path, new Uint8Array(await blob.arrayBuffer()));
}

// ???
// allScriptsDo(fn, ?target)

// == a bunch of useful getters ==
// blocks.getBlock(bid): ?block
// blocks.getScripts(): [bid]
// blocks.getNextBlock(): bid?
// blocks.getBranch(bid, num): bid?

// what's a race condition when you can just
const awa = () => new Promise(resolve => setTimeout(resolve, 0));

const main = async () => {
    await load("example/cg.sb3");
    vm.start();
    core.main();

    // let id = "";
    // vm.runtime.allScriptsDo(newId => {id = newId}, vm.runtime.getEditingTarget()!);
    // console.log(vm.runtime.getEditingTarget()?.blocks.getBlock(id));

    // const events = [
    //     {
    //         type: "create",
    //         blockId: "fhui31qedkfjs",
    //         xml: {
    //             innerHtml: "xml string"
    //         }
    //     },
    //     {
    //         type: "change",
    //         blockId: "blockId",
    //         element: "field | comment | collapsed | disabled | inline | mutation",
    //         name: "field name if element == field",
    //         newValue: "newValue",
    //     },
    //     {
    //         type: "move",
    //         blockId: "blockId",
    //         oldParentId: "oldParentId",
    //         oldInputName: "oldInputName",
    //         newParentId: "newParentId",
    //         newInputName: "newInputName",
    //         newCoordinate: "newCoordinate",
    //     },
    //     {
    //         type: "delete",
    //         blockId: "blockId",
    //     },
    // ];

    workspace.newBlock("hiya", "foobars");
    await awa();
    
    console.log(vm.runtime.targets.map(target => target.blocks.getBlock("foobars")).find(v => v !== undefined));

    save("example/cg2.sb3");
    vm.quit();
}

main()
