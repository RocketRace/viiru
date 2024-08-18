/* eslint-disable @typescript-eslint/no-explicit-any */
// here be any-casts

// looking at the dependencies, we've got
import fs from 'node:fs';
// utilities
import core from 'viiru_core';
// the custom editor logic
import VM from 'scratch-vm';
// the scratch core VM
import Storage from 'scratch-storage';
// serialization for scratch assets
import { JSDOM } from 'jsdom';
const jsdom = new JSDOM();
(global as any).window = jsdom.window;
global.document = window.document;
global.DOMParser = window.DOMParser;
global.XMLSerializer = window.XMLSerializer;
import SB from 'scratch-blocks';
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
// problem: it's a whole-ass UI designed for the browser, and therefore
// uses all sorts of browser-only globals just to initialize
// 
// so that's that I'm doing, dumping browser-only globals into the NodeJS
// runtime. a huge dependency tree never hurt anybody
const vm = new VM();
vm.attachStorage(new Storage())
// yeah I'm pretending to be a browser. don't worry about it
const fakeDiv = document.createElement('div');
document.body.appendChild(fakeDiv);
fakeDiv.setAttribute('id', 'fakeDiv');
fakeDiv.setAttribute('style', 'height: 480px; width: 600px;');
const workspace = (SB as any).inject('fakeDiv', {});
// one interesting this about the workspace is that it's entirely
// unsynchronized with the VM. its only purpose is to create
// block events that I can then pass through to the VM.

// == interface functions ==
const loadProject = async (path: string) => {
    const buffer = fs.readFileSync(path);
    await vm.loadProject(buffer);
}

const saveProject = async (path: string) => {
    const blob = await vm.saveProjectSb3();
    fs.writeFileSync(path, new Uint8Array(await blob.arrayBuffer()));
}

// this one is weird because the VM will only accept fully 
// fledged block data structures (and I don't want to reimplement
// every single scratch block), but the templates are only implemented
// by scratch-blocks in its own internal representation format.
// so I manually call the VM's event listener, pretending to be scratch-blocks
const createBlock = (opcode: string, id?: string) => {
    const block = workspace.newBlock(opcode, id);
    const event = new (SB as any).Events.Create(block);
    (vm as any).blockListener(event);
}

const deleteBlock = (id: string) => {
    (vm.runtime.getEditingTarget()?.blocks as any).deleteBlock(id)
}

// ???
// allScriptsDo(fn, ?target)

// == a bunch of useful getters ==
// blocks.getBlock(bid): ?block
// blocks.getScripts(): [bid]
// blocks.getNextBlock(): bid?
// blocks.getBranch(bid, num): bid?

// what's a race condition when you can just
// const awa = (it?: number) => new Promise(resolve => setTimeout(resolve, it ?? 0));

const main = async () => {
    await loadProject("example/cg.sb3");
    vm.start();
    core.main();

    // const events = [
    //     {
    //         type: "create",
    //         blockId: "fhui31qedkfjs",
    //         xml: {
    //             outerHtml: "xml string (SUCKS, don't like it)"
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

    createBlock('control_forever', 'yippee');
    console.log(vm.runtime.getEditingTarget()?.blocks.getBlock('yippee'));
    deleteBlock('yippee');
    console.log(vm.runtime.getEditingTarget()?.blocks.getBlock('yippee'));
    
    saveProject("example/cg2.sb3");
    vm.quit();
}

main()
