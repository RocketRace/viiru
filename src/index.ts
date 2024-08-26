/* eslint-disable @typescript-eslint/no-explicit-any */
// here be any-casts

// looking at the dependencies, we have
import fs from 'node:fs';
import { loopWhile } from 'deasync';
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
global.MouseEvent = window.MouseEvent;
import SB from 'scratch-blocks';
// const makeToolboxXML = require("./vendored/make-toolbox-xml");
// ... uhhhhhhhhh so
//
// scratch-blocks contains an UI implementation for the scratch block 
// editor and the VM listens to specifically its events, and it also
// offers convenience methods to interact with the VM's blocks (which
// is exactly what this project is, a block editor).
//
// but those block events are a pain in the ass to construct manually -
// like, writing-template-xml-strings-in-json kind of a pain -
// so I'd much rather pull in the dependency.
//
// problem: it's a whole-ass UI designed for the browser, and therefore
// uses all sorts of browser-only globals just to initialize.
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
// this can theoretically cause issues with conflicting IDs...
// but that's rare enough that I'll ignore it completely.

// yes, this is ugly, but I do NOT want to deal with callback resolution
// *across an ABI boundary*
const resolve = <T>(p: Promise<T>): T | undefined => {
    let result: T | undefined = undefined;
    let done = false;
    p.then(res => result = res).finally(() => done = true);
    loopWhile(() => !done);
    return result;
}

// == editor functions ==
const loadProject = (path: string): boolean => {
    try {
        const buffer = fs.readFileSync(path);
        resolve(vm.loadProject(buffer));
        return true;
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (error: any) {
        return false;
    }
}

const saveProject = (path: string): boolean => {
    try {
        resolve(vm.saveProjectSb3().then(
            blob => blob.arrayBuffer().then(
                buffer => fs.writeFileSync(path, new Uint8Array(buffer))
            )
        ));
        return true;
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (error: any) {
        return false;
    }
}

// this one is a bit weird because the VM will only accept fully 
// fledged block JSON structures (and I don't want to reimplement
// every single scratch block), but the templates are only implemented
// by scratch-blocks in its own internal representation format.
// so I manually call the VM's event listener, pretending to be scratch-blocks,
// post-initializing the block with some extra data.
const createBlock = (opcode: string, isShadow: boolean, id?: string) => {
    const block = workspace.newBlock(opcode, id);
    if (isShadow) {
        // blocks behave goofy otherwise
        block.setShadow(true);
    }
    const event = new (SB as any).Events.BlockCreate(block);
    (vm as any).blockListener(event);
    vm.runtime.getEditingTarget()?.blocks.getBlock(block.id);
    return block.id;
}

const deleteBlock = (id: string) => {
    (vm.runtime.getEditingTarget()?.blocks as any).deleteBlock(id);
}

// the various moveBlock routines have been split for each usecase
const slideBlock = (id: string, x: number, y: number) => {
    (vm.runtime.getEditingTarget()?.blocks as any).moveBlock({
        id,
        newCoordinate: { x, y },
    });
}

const attachBlock = (id: string, newParent: string, newInput: string | undefined, isShadow: boolean) => {
    const existingParent = vm.runtime.getEditingTarget()?.blocks.getBlock(newParent)?.next;
    if (existingParent) {
        return;
    }
    const block = vm.runtime.getEditingTarget()?.blocks.getBlock(id);
    if (block) {
        block.shadow = isShadow;
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
// TODO: value is an object!
// { id?: string, name: string, value: string, variableType?: string }
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

const getAllBlocks = (): Record<string, object> => 
    vm.runtime.getEditingTarget()?.blocks._blocks ?? {}

const getVariablesOfType = (type: "" | "list" | "broadcast_msg"): Record<string, string> => {
    const output: Record<string, string> = {}
    const vars = vm.runtime.getEditingTarget()?.variables ?? {};
    Object.keys(vars).forEach(key => {
        if (vars[key].type == type) {
            output[key] = vars[key].name;
        }
    })
    return output
}

const API = {
    loadProject,
    saveProject,
    createBlock,
    deleteBlock,
    slideBlock,
    attachBlock,
    detachBlock,
    changeField,
    changeMutation,
    getAllBlocks,
    getVariablesOfType,
}

const main = async () => {
    vm.start();
    core.main(API);
    vm.quit();
}

main()
