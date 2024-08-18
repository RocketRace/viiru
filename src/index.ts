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

// == editor functions ==
const loadProject = async (path: string) => {
    const buffer = fs.readFileSync(path);
    await vm.loadProject(buffer);
}

const saveProject = async (path: string) => {
    const blob = await vm.saveProjectSb3();
    fs.writeFileSync(path, new Uint8Array(await blob.arrayBuffer()));
}

// this one is a bit weird because the VM will only accept fully 
// fledged block data structures (and I don't want to reimplement
// every single scratch block), but the templates are only implemented
// by scratch-blocks in its own internal representation format.
// so I manually call the VM's event listener, pretending to be scratch-blocks
const createBlock = (opcode: string, id?: string) => {
    const block = workspace.newBlock(opcode, id);
    const event = new (SB as any).Events.Create(block);
    (vm as any).blockListener(event);
    const createdBlock = vm.runtime.getEditingTarget()?.blocks.getBlock(block.id);
    if (createdBlock) {
        block.inputList.forEach((input: any) => {
            if (input.name !== '') {
                createdBlock.inputs[input.name] = {
                    name: input.name,
                    block: null as any,
                    shadow: null
                }
            }
        });
    }
}

const deleteBlock = (id: string) => {
    (vm.runtime.getEditingTarget()?.blocks as any).deleteBlock(id)
}

// the various moveBlock routines have been split for each usecase
const slideBlock = (id: string, x: number, y: number) => {
    (vm.runtime.getEditingTarget()?.blocks as any).moveBlock({
        id,
        newCoordinate: {x, y},
    });
}

const attachBlock = (id: string, newParent: string, newInput?: string) => {
    const existingParent = vm.runtime.getEditingTarget()?.blocks.getBlock(newParent)?.next;
    if (existingParent) {
        return;
    }
    (vm.runtime.getEditingTarget()?.blocks as any).moveBlock({
        id,
        newParent,
        newInput
    });
}

const detachBlock = (id: string) => {
    const oldParent = vm.runtime.getEditingTarget()?.blocks.getBlock(id)?.parent;
    if (!oldParent) {
        return;
    }
    // first try to unslot it from any input slot
    const inputs: any = vm.runtime.getEditingTarget()?.blocks.getBlock(oldParent)?.inputs;
    if (inputs) {
        for (const oldInput in inputs) {
            if (Object.prototype.hasOwnProperty.call(inputs, oldInput)) {
                if (inputs[oldInput].block == id) {
                    (vm.runtime.getEditingTarget()?.blocks as any).moveBlock({
                        id,
                        oldParent,
                        oldInput,
                    });
                    return;
                }
            }
        }
    }
    // otherwise, fall back to removing it from its parent
    (vm.runtime.getEditingTarget()?.blocks as any).moveBlock({
        id,
        oldParent,
    });
}

// the changeBlock routines were also split apart
const changeField = (id: string, name: string, value: string) => {
    // VARIABLE, LIST, or BROADCAST_OPTION, or variably named dropdown inputs
    (vm.runtime.getEditingTarget()?.blocks as any).changeBlock({
        id,
        element: 'field',
        name,
        value
    });
}

// todo: is this properly implemented?
const changeMutation = (id: string, value: any) => {
    (vm.runtime.getEditingTarget()?.blocks as any).changeBlock({
        id,
        element: 'mutation',
        value
    });
}

// todo: is this needed? the frontend doesn't care for monitors
const changeCheckbox = (id: string, value: boolean) => {
    (vm.runtime.getEditingTarget()?.blocks as any).changeBlock({
        id,
        element: 'checkbox',
        value
    });
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
    (vm.runtime.getEditingTarget()?.blocks as any).deleteAllBlocks();

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

    createBlock('event_whenflagclicked', 'starting');
    slideBlock('starting', 35, 35);
    createBlock('control_if', 'if');
    createBlock('looks_sayforsecs', 'speak');
    attachBlock('if', 'starting');
    attachBlock('speak', 'if', 'SUBSTACK');
    createBlock('operator_equals', 'cond!!');
    attachBlock('cond!!', 'if', 'CONDITION');
    console.log(vm.runtime.getEditingTarget()?.blocks.getBlock('starting'));
    console.log(vm.runtime.getEditingTarget()?.blocks.getBlock('if'));
    console.log(vm.runtime.getEditingTarget()?.blocks.getBlock('cond!!'));
    console.log(vm.runtime.getEditingTarget()?.blocks.getBlock('speak'));
    
    saveProject("example/cg2.sb3");
    vm.quit();
}

main()
