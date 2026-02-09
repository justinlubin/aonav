import { JSDOM } from "jsdom";

const dom = new JSDOM(`<!DOCTYPE html><html><head></head><body></body></html>`);

(global as any).window = dom.window;
(global as any).document = dom.window.document;
(global as any).HTMLElement = dom.window.HTMLElement;
(global as any).Element = dom.window.Element;
(global as any).Node = dom.window.Node;
(global as any).CustomEvent = dom.window.CustomEvent;
(global as any).Event = dom.window.Event;
(global as any).MouseEvent = dom.window.MouseEvent;
(global as any).Document = dom.window.Document;
(global as any).HTMLDocument = dom.window.HTMLDocument;

Object.defineProperty(global, 'navigator', {
  value: { userAgent: "node.js" },
  configurable: true,
});

(global as any).MutationObserver = class {
  constructor(callback: Function) {}
  disconnect() {}
  observe(target: any, options?: any) {}
};

(global as any).window.matchMedia = (query: string) => ({
  matches: false,
  media: query,
  onchange: null,
  addListener: () => {},
  removeListener: () => {},
  addEventListener: () => {},
  removeEventListener: () => {},
  dispatchEvent: () => false,
});

(global as any).CSSStyleSheet = class { constructor() {} };

(global as any).requestAnimationFrame = (callback: FrameRequestCallback) =>
  setTimeout(callback, 0);
(global as any).cancelAnimationFrame = (id: NodeJS.Timeout) => clearTimeout(id);

(global as any).customElements = {
  define: (_name: string, _constructor: any) => {},
  get: (_name: string) => undefined,
  whenDefined: (_name: string) => Promise.resolve(),
};

// Polyfill getComputedStyle
(global as any).getComputedStyle = (elt: any) => ({
  getPropertyValue: (_prop: string) => "",
});

(global as any).localStorage = {
  getItem: () => null,
  setItem: () => {},
};

import fs from 'fs';
import path from 'path';
import React from 'react';
import ReactDOMServer from 'react-dom/server';

import TreeInfo from "@argus/common/TreeInfo";
import { unpackProofNode } from "@argus/common/TreeInfo";
import { TyCtxt } from "@argus/print/context";
import { PrintGoal, PrintImplHeader } from "@argus/print/lib";

type SerializedTree = any;

async function main() {

  const args = process.argv.slice(2);
  if (args.length < 2) {
    console.error('Usage: ts-node scripts/render_tree_with_treeinfo.ts tree.json out.json');
    process.exit(2);
  }
  const [treePath, outPath] = args;
  const absTree = path.resolve(treePath);
  const raw = fs.readFileSync(absTree, 'utf8');
  const parsed = JSON.parse(raw);

  // get serialized tree
  let tree: SerializedTree | undefined = undefined;
  if (Array.isArray(parsed)) {
    tree = parsed.find((x: any) => x && typeof x === 'object' && x.goals !== undefined) || parsed[0];
  } else if (parsed && typeof parsed === 'object') {
    tree = parsed;
  }
  if (!tree) {
    console.error('Could not find a SerializedTree inside', treePath);
    process.exit(3);
  }

  // make TreeInfo
  const treeInfo = TreeInfo.new(tree, false);
  if (!treeInfo) {
    console.error('TreeInfo.new returned undefined');
    process.exit(4);
  }

 const typeCtx = {
  interner: tree.tys ?? [],
  projections: tree.projection_values ?? {}
  };

  // Helper to render a React element to static HTML inside TyCtxt provider
  function renderWithTyCtx(element: React.ReactElement) {
    const wrapped = React.createElement(TyCtxt.Provider as any, { value: typeCtx }, element as any);
    try {
      // render to static HTML, then expand any <details> toggles so the
      // hidden children are included in the extracted text.
      const rawHtml = ReactDOMServer.renderToStaticMarkup(wrapped);
      try {
        const tmp = document.createElement('div');
        tmp.innerHTML = rawHtml;
        // find all details and replace the <details> element with its contents
        tmp.querySelectorAll('details').forEach((d: any) => {
          // extract inner content (children after <summary>)
          const summary = d.querySelector('summary');
          // clone children except summary
          const frag = document.createDocumentFragment();
          Array.from(d.childNodes).forEach((child: any) => {
            if (child !== summary) frag.appendChild(child.cloneNode(true));
          });
          d.parentNode?.replaceChild(frag, d);
        });
        return extractPlainTextFromHtml(tmp.innerHTML);
      } catch (e:any) {
        return extractPlainTextFromHtml(rawHtml);
      }
    } catch (e:any) {
  // include stack if available to aid debugging
  const stack = e && e.stack ? `\n${String(e.stack)}` : "";
  return `<error>${String(e.message)}${stack}</error>`;
    }
  }

  // small helper: extract plain text from static HTML
  function extractPlainTextFromHtml(htmlString: string) {
    try {
      const tmp = document.createElement('div');
      tmp.innerHTML = htmlString;

      // handle specially formatted interspersed lists (CommaSeparated / PlusSeparated)
      const interspersed = tmp.querySelectorAll('.interspersed-list');
      interspersed.forEach(node => {
        // collect the text of each child dsp (display:inline-block)
        const parts: string[] = [];
        node.querySelectorAll(':scope > span').forEach((child: any, idx) => {
          // child may contain the element rendered; use its textContent
          const txt = (child.textContent || '').trim();
          parts.push(txt);
        });

        // determine separator based on child class
        let sep = ', ';
        if (node.querySelector('.plus')) sep = ' + ';

        // replace node's text with joined version
        const joined = parts.join(sep);
        const repl = document.createTextNode(joined);
        node.parentNode?.replaceChild(repl, node);
      });

  // now return the flattened text, and normalize NBSP (\u00A0) to regular space
  const raw = tmp.textContent || tmp.innerText || '';
  return raw.replace(/\u00A0/g, ' ');
    } catch (e:any) {
      return '';
    }
  }

  const out: any = { root: tree.root.toString(), goals: {}, candidates: {}, topology: {}, yesGoals: {} };

  const new_topology = new Map();
  const yesses = [];

  // keep track of which nodes are not filtered out
  const keptNodes: Set<number> = recurseChildren(treeInfo.root, new Set([treeInfo.root]));
  console.log("final kept nodes:");
  console.log(new_topology);
  for (const key of new_topology.keys()) {
    out.topology[key] = [...new_topology.get(key)].map(v => v.toString());
  }

    // function recurseChildren(id: number, keptSoFar: Set<number>): Set<number> {
    //   //console.log("here");
    //   let curr = id;
    //   let children = treeInfo?.children(curr);
    //   if (children == undefined) {
    //     throw Error();
    //   }
    //   addToTopology(curr, [...children]);
    //   while (children.length != 0) {
    //     //console.log(children);
    //     const curr_maybe = children.pop();
    //     if (!curr_maybe) {
    //       throw Error();
    //     }
    //     curr = curr_maybe
    //     keptSoFar.add(curr)
    //     const new_children = treeInfo?.children(curr);
    //     if (new_children) {
    //       addToTopology(curr, new_children);
    //       for (const child of new_children) {
    //         if (!keptSoFar.has(child)) {
    //           keptSoFar.add(child)
    //           children.push(child)
    //         }
    //       }
    //     }
    //   }
    //   return keptSoFar;
    // }
  

  // get all nodes that haven't been filtered out (should start from root)
  // add children for each id to out.topology- if id not a key in topology yet, add new set
   function recurseChildren(id: number, keptSoFar: Set<number>): Set<number> {
  //   // console.log("\n");
  //   // console.log("parent: " + id);
     const children = treeInfo?.children(id);

     if (children?.length == 0) {
       // base case: leaf node
       return keptSoFar;
     } else {
       // recursive case: add to kept and call on each new child
       if (children != undefined) {
         addToTopology(id, children);
         for (const child of children) {
          if (!keptSoFar.has(child)) {
            keptSoFar.add(child);
           recurseChildren(child, keptSoFar)?.forEach(i => keptSoFar.add(i));
          }
           // add to kept and recurse
           
         }
         return keptSoFar;
       } else {
         throw Error("Children undefined");
       }
     }
   }

  // mutates topology
  // if parent not in topology, add parent -> set containing children to topology
  // if parent in topology, append children to existing set
  function addToTopology(parent: number, children: number[]) {
    if (!new_topology.has(parent)) {
      new_topology.set(parent, new Set<number>);
    }
    const existing_children = new_topology.get(parent);
    if (existing_children != undefined) {
      children.forEach(child => existing_children.add(child));
      new_topology.set(parent, existing_children);
    } else {
      throw Error("Existing children should not be undefined");
    }
  }

  // Goals: iterate through tree.goals array
  if (Array.isArray(tree.goals)) {
    for (let i = 0; i < tree.goals.length; i++) {
      if (!keptNodes.has(i)) {
        continue;
      }
      const goal = tree.goals[i];
      // PrintGoal component expects a GoalData object as { value: ... }
      // In the repo, PrintGoal accepts { o: GoalData }
      try {
        const node = unpackProofNode(i);
        if ("Goal" in node) {
          const html = renderWithTyCtx(React.createElement(PrintGoal as any, { o: goal }));
          out.goals[i] = html;
        }
        // collect yesses
          const res = tree.results[goal.result];
          if (res === 'yes') {
            yesses.push(i);
          }
      } catch (e:any) {
        out.goals[i] = { error: String(e) };
      }
    }
  }
  out.yesGoals = yesses.map(x => x.toString());

  // Candidates: serialized tree has `candidates` array; each entry can be Any | Impl | ParamEnv
  if (Array.isArray(tree.candidates)) {
    for (let j = 0; j < tree.candidates.length; j++) {
      const cand = tree.candidates[j];
      if (cand === undefined) {
        continue;
      }
      // inverse of `node & ((1 << 30) - 1)` used in unpackProofNode:
      // set the candidate flag (bit 31) on the 30-bit index to form a ProofNode
      const i = ((j & 0x3fffffff) | 0x80000000) >>> 0;
      if (!keptNodes.has(i)) {
        continue;
      }
      try {
        // Impl variant shape in frontend is candidate.Impl.hd
        if ('Impl' in cand) {
          const implHd = cand.Impl.hd;
          const html = renderWithTyCtx(React.createElement(PrintImplHeader as any, { impl: implHd }));
          out.candidates[i] = html;
        } //else if ('Any' in cand) {
         // out.candidates[i] = { type: 'Any', value: cand.Any };
       // } else if ('ParamEnv' in cand) {
         // out.candidates[i] = { type: 'ParamEnv' };
       // } else {
          //out.candidates[i] = { type: 'Unknown', raw: cand };
        //}
      } catch (e:any) {
        out.candidates[i] = { error: String(e) };
      }
    }
  }

  fs.writeFileSync(outPath, JSON.stringify(out), 'utf8');
}

main().catch(err => { console.error(err); process.exit(1); });
