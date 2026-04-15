/**
 * Generate a deterministic identicon SVG data URI from a pubkey string.
 * Produces a 5x5 symmetric grid of colored cells on a background.
 */

const PALETTE = [
  "#f38ba8", // red
  "#fab387", // peach
  "#f9e2af", // yellow
  "#a6e3a1", // green
  "#94e2d5", // teal
  "#89b4fa", // blue
  "#b4befe", // lavender
  "#cba6f7", // mauve
  "#f5c2e7", // pink
  "#74c7ec", // sapphire
];

function hashBytes(pubkey: string): number[] {
  const bytes: number[] = [];
  for (let i = 0; i < pubkey.length; i += 2) {
    const hex = pubkey.slice(i, i + 2);
    const val = parseInt(hex, 16);
    if (!isNaN(val)) {
      bytes.push(val);
    }
  }
  // Ensure at least 16 bytes for the grid
  while (bytes.length < 16) {
    bytes.push(0);
  }
  return bytes;
}

export function generateIdenticon(pubkey: string, size: number = 64): string {
  const bytes = hashBytes(pubkey);

  const fgColor = PALETTE[bytes[0] % PALETTE.length];
  const bgLightness = 25 + (bytes[1] % 15);
  const bgColor = `hsl(${(bytes[2] * 3) % 360}, 15%, ${bgLightness}%)`;

  // 5x5 grid, mirrored horizontally (only need 3 columns)
  const cells: boolean[][] = [];
  for (let row = 0; row < 5; row++) {
    cells[row] = [];
    for (let col = 0; col < 3; col++) {
      const byteIdx = 3 + row * 3 + col;
      cells[row][col] = bytes[byteIdx % bytes.length] % 2 === 0;
    }
    // Mirror
    cells[row][3] = cells[row][1];
    cells[row][4] = cells[row][0];
  }

  const cellSize = size / 5;
  let rects = "";
  for (let row = 0; row < 5; row++) {
    for (let col = 0; col < 5; col++) {
      if (cells[row][col]) {
        rects += `<rect x="${col * cellSize}" y="${row * cellSize}" width="${cellSize}" height="${cellSize}" fill="${fgColor}"/>`;
      }
    }
  }

  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}"><rect width="${size}" height="${size}" fill="${bgColor}"/>${rects}</svg>`;

  return `data:image/svg+xml;base64,${btoa(svg)}`;
}
