// File-icon resolver — direct port of GitButler's getFileIcon helper
// (https://github.com/gitbutlerapp/gitbutler/blob/master/packages/ui/src/lib/components/file/icon/getFileIcon.ts).
// All SVGs in ./svg are inlined eagerly via Vite's import.meta.glob so we get
// raw markup we can feed to {@html} in a Svelte component.

import {
  symbolFileExtensionsToIcons,
  symbolFileNamesToIcons,
} from './typeMap';

const modules = import.meta.glob<string>('./svg/*.svg', {
  query: '?raw',
  import: 'default',
  eager: true,
});

export const fileIcons: Record<string, string> = {};
for (const [modulePath, svg] of Object.entries(modules)) {
  const name = modulePath.replace(/^.*\//, '').replace(/\.svg$/, '');
  fileIcons[name] = svg;
}

export function getFileIcon(fileName: string): string {
  fileName = fileName.toLowerCase();

  // Direct match (e.g. an exact file like "dockerfile" maps to a same-named icon).
  if (fileIcons[fileName]) return fileIcons[fileName];

  const splitName = fileName.split('.');
  let iconName = '';

  // Walk progressively shorter suffixes: "foo.spec.ts" → "foo.spec.ts" → "spec.ts" → "ts".
  while (splitName.length) {
    const curName = splitName.join('.');
    if (symbolFileNamesToIcons[curName]) {
      iconName = symbolFileNamesToIcons[curName] ?? '';
      break;
    }
    if (symbolFileExtensionsToIcons[curName]) {
      iconName = symbolFileExtensionsToIcons[curName] ?? '';
      break;
    }
    splitName.shift();
  }

  if (iconName === '') iconName = 'document';
  return fileIcons[iconName] ?? fileIcons.document ?? '';
}
