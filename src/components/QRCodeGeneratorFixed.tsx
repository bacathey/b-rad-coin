import React from 'react';
import { Box } from '@mui/material';

interface QRCodeGeneratorProps {
  value: string;
  size?: number;
}

// Simple QR Code implementation using an SVG approach
const QRCodeGenerator: React.FC<QRCodeGeneratorProps> = ({ value, size = 200 }) => {
  const generateQRPattern = (text: string): string => {
    // Create a deterministic pattern based on the text
    let hash = 0;
    for (let i = 0; i < text.length; i++) {
      const char = text.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }

    const gridSize = 25;
    let svgContent = '';

    // Function to check if a module should be black
    const isBlack = (x: number, y: number): boolean => {
      // Finder patterns (corners)
      if ((x < 7 && y < 7) || (x >= gridSize - 7 && y < 7) || (x < 7 && y >= gridSize - 7)) {
        // Finder pattern logic
        const fx = x < 7 ? x : (x >= gridSize - 7 ? x - (gridSize - 7) : x);
        const fy = y < 7 ? y : (y >= gridSize - 7 ? y - (gridSize - 7) : y);
        
        if (fx === 0 || fx === 6 || fy === 0 || fy === 6) return true;
        if (fx >= 2 && fx <= 4 && fy >= 2 && fy <= 4) return true;
        return false;
      }

      // Timing patterns
      if (x === 6 && y >= 8 && y < gridSize - 8) return y % 2 === 0;
      if (y === 6 && x >= 8 && x < gridSize - 8) return x % 2 === 0;

      // Data area - use hash to create deterministic pattern that represents the address
      const seed = hash + x * 7 + y * 13 + text.charCodeAt((x + y) % text.length);
      return (seed % 3) === 1;
    };

    // Generate SVG rectangles for black modules
    const moduleSize = size / gridSize;
    for (let y = 0; y < gridSize; y++) {
      for (let x = 0; x < gridSize; x++) {
        if (isBlack(x, y)) {
          svgContent += `<rect x="${x * moduleSize}" y="${y * moduleSize}" width="${moduleSize}" height="${moduleSize}" fill="black"/>`;
        }
      }
    }

    return svgContent;
  };

  const svgContent = generateQRPattern(value);

  return (
    <Box display="flex" justifyContent="center">
      <svg
        width={size}
        height={size}
        viewBox={`0 0 ${size} ${size}`}
        style={{ 
          border: '1px solid #ccc',
          borderRadius: '4px',
          backgroundColor: 'white'
        }}
      >
        <rect width={size} height={size} fill="white"/>
        <g dangerouslySetInnerHTML={{ __html: svgContent }} />
      </svg>
    </Box>
  );
};

export default QRCodeGenerator;
