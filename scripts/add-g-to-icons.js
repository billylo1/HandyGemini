#!/usr/bin/env node

/**
 * Script to add a "G" badge to app icons to differentiate HandyGemini from Handy
 */

import sharp from 'sharp';
import { readdir, stat } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const iconsDir = join(__dirname, '../src-tauri/icons');

// Icon files to process (main icons that need the G badge)
const mainIcons = [
  'icon.png',
  '32x32.png',
  '64x64.png',
  '128x128.png',
  '128x128@2x.png',
  'logo.png',
];

// Square logo files
const squareLogos = [
  'Square30x30Logo.png',
  'Square44x44Logo.png',
  'Square71x71Logo.png',
  'Square89x89Logo.png',
  'Square107x107Logo.png',
  'Square142x142Logo.png',
  'Square150x150Logo.png',
  'Square284x284Logo.png',
  'Square310x310Logo.png',
  'StoreLogo.png',
];

async function addGToIcon(iconPath) {
  try {
    const image = sharp(iconPath);
    const metadata = await image.metadata();
    const { width, height } = metadata;

    // Calculate badge size (about 30% of the smaller dimension)
    const badgeSize = Math.min(width, height) * 0.3;
    const badgePadding = badgeSize * 0.1;
    const fontSize = badgeSize * 0.6;
    
    // Position badge in bottom-right corner
    const badgeX = width - badgeSize - badgePadding;
    const badgeY = height - badgeSize - badgePadding;

    // Create SVG for the "G" badge
    const svg = `
      <svg width="${width}" height="${height}" xmlns="http://www.w3.org/2000/svg">
        <!-- Badge background circle -->
        <circle cx="${badgeX + badgeSize / 2}" cy="${badgeY + badgeSize / 2}" 
                r="${badgeSize / 2 - 2}" 
                fill="#4285F4" 
                stroke="#FFFFFF" 
                stroke-width="3"/>
        <!-- G letter -->
        <text x="${badgeX + badgeSize / 2}" 
              y="${badgeY + badgeSize / 2}" 
              font-family="Arial, sans-serif" 
              font-size="${fontSize}" 
              font-weight="bold" 
              fill="#FFFFFF" 
              text-anchor="middle" 
              dominant-baseline="central">G</text>
      </svg>
    `;

    // Composite the badge onto the original image
    const output = await image
      .composite([
        {
          input: Buffer.from(svg),
          top: 0,
          left: 0,
        },
      ])
      .toBuffer();

    // Write the modified image back
    await sharp(output).toFile(iconPath);
    console.log(`✓ Added G badge to ${iconPath}`);
  } catch (error) {
    console.error(`✗ Error processing ${iconPath}:`, error.message);
  }
}

async function processIcons() {
  console.log('Adding "G" badge to HandyGemini icons...\n');

  // Process main icons
  for (const icon of mainIcons) {
    const iconPath = join(iconsDir, icon);
    try {
      await stat(iconPath);
      await addGToIcon(iconPath);
    } catch (error) {
      if (error.code === 'ENOENT') {
        console.log(`⚠ Skipping ${icon} (not found)`);
      } else {
        console.error(`✗ Error with ${icon}:`, error.message);
      }
    }
  }

  // Process square logos
  for (const logo of squareLogos) {
    const logoPath = join(iconsDir, logo);
    try {
      await stat(logoPath);
      await addGToIcon(logoPath);
    } catch (error) {
      if (error.code === 'ENOENT') {
        console.log(`⚠ Skipping ${logo} (not found)`);
      } else {
        console.error(`✗ Error with ${logo}:`, error.message);
      }
    }
  }

  console.log('\n✓ Icon processing complete!');
  console.log('Note: You may need to regenerate .icns and .ico files for macOS/Windows.');
}

processIcons().catch(console.error);
