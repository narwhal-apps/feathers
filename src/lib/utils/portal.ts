/** Svelte action that moves the node to `<body>` on mount and back on
 *  destroy. Used by overlay components (modals, popovers) so their backdrop
 *  + content sit at the root stacking context — otherwise a parent with
 *  `z-index` (e.g. the titlebar) traps them and later siblings paint over
 *  the backdrop.
 *
 *  Event listeners attached inside the node are preserved (Svelte uses
 *  delegated handlers on the element itself, not the original parent).
 */
export function portal(node: HTMLElement) {
  const target = document.body;
  target.appendChild(node);
  return {
    destroy() {
      if (node.parentNode === target) {
        target.removeChild(node);
      }
    },
  };
}
