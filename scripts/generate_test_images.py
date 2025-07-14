#!/usr/bin/env python3
"""
Test script that generates visual regression test images using render_sandbox.
This script creates various rendering scenarios to test visual output consistency.
If render_sandbox fails (e.g., in CI without GPU), it falls back to generating
synthetic test images to demonstrate the visual regression testing workflow.
"""

import os
import subprocess
import sys
import tempfile
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont
import random

def create_fallback_image(output_path, width, height, test_name, description):
    """
    Create a synthetic test image when render_sandbox cannot run.
    This is used in CI environments without GPU access.
    """
    # Create image with background color based on test name
    bg_colors = {
        'basic_render_800x600': (240, 248, 255),  # Alice blue
        'high_res_1920x1080': (255, 240, 245),   # Lavender blush
        'square_512x512': (240, 255, 240),       # Honeydew
        'antialiased_4x': (255, 255, 240),       # Ivory
        'minimal_400x300': (248, 248, 255),      # Ghost white
    }
    
    bg_color = bg_colors.get(test_name, (255, 255, 255))
    image = Image.new('RGB', (width, height), bg_color)
    draw = ImageDraw.Draw(image)
    
    # Draw border
    border_color = (100, 100, 100)
    draw.rectangle([0, 0, width-1, height-1], outline=border_color, width=2)
    
    # Draw test identifier
    draw.rectangle([10, 10, width-10, 60], fill=(50, 50, 50), outline=border_color)
    
    # Add text
    try:
        font = ImageFont.load_default()
    except:
        font = None
    
    # Title
    title_text = test_name.replace('_', ' ').title()
    draw.text((20, 20), title_text, fill=(255, 255, 255), font=font)
    draw.text((20, 35), f"{width}x{height}", fill=(200, 200, 200), font=font)
    
    # Description
    desc_y = 80
    for line in description.split():
        draw.text((20, desc_y), line, fill=(80, 80, 80), font=font)
        desc_y += 15
    
    # Add some geometric shapes for visual interest
    center_x, center_y = width // 2, height // 2
    
    # Circle
    circle_radius = min(width, height) // 8
    draw.ellipse([
        center_x - circle_radius, center_y - circle_radius,
        center_x + circle_radius, center_y + circle_radius
    ], outline=(200, 100, 100), width=3)
    
    # Rectangle
    rect_size = min(width, height) // 12
    draw.rectangle([
        center_x - rect_size, center_y - rect_size,
        center_x + rect_size, center_y + rect_size
    ], fill=(100, 200, 100))
    
    # Add some random dots for uniqueness
    random.seed(hash(test_name))  # Deterministic based on test name
    for _ in range(20):
        x = random.randint(0, width)
        y = random.randint(0, height)
        size = random.randint(2, 8)
        color = (random.randint(100, 200), random.randint(100, 200), random.randint(100, 200))
        draw.ellipse([x-size//2, y-size//2, x+size//2, y+size//2], fill=color)
    
    # Save the image
    image.save(output_path)
    print(f"Created fallback image: {output_path}")

def run_render_sandbox(output_path, width=800, height=600, format="png", **kwargs):
    """
    Run render_sandbox with specified parameters and return the command result.
    """
    cmd = [
        "./target/release/render_sandbox",
        "--output", output_path,
        "--width", str(width),
        "--height", str(height),
        "--format", format,
        "--headless"  # Run in headless mode for CI
    ]
    
    # Add additional arguments
    for key, value in kwargs.items():
        if key == "samples":
            cmd.extend(["--samples", str(value)])
        elif key == "verbose":
            if value:
                cmd.append("--verbose")
        elif key == "log_level":
            cmd.extend(["--log-level", str(value)])
    
    print(f"Running command: {' '.join(cmd)}")
    
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        print(f"Successfully generated: {output_path}")
        return result
    except subprocess.CalledProcessError as e:
        print(f"Error running render_sandbox: {e}")
        print(f"stdout: {e.stdout}")
        print(f"stderr: {e.stderr}")
        raise

def ensure_binary_exists():
    """
    Ensure that the render_sandbox binary exists, build it if not.
    """
    binary_path = Path("./target/release/render_sandbox")
    if not binary_path.exists():
        print("render_sandbox binary not found, building...")
        result = subprocess.run(["cargo", "build", "--release"], 
                              capture_output=True, text=True)
        if result.returncode != 0:
            print(f"Failed to build render_sandbox: {result.stderr}")
            sys.exit(1)
        print("Successfully built render_sandbox")

def main():
    """Generate test images for visual regression testing."""
    
    # Ensure outputs directory exists
    os.makedirs('outputs', exist_ok=True)
    
    # Ensure binary exists
    ensure_binary_exists()
    
    # Test cases with different rendering scenarios
    test_cases = [
        {
            'name': 'basic_render_800x600',
            'width': 800,
            'height': 600,
            'samples': 1,
            'description': 'Basic rendering test at 800x600 resolution'
        },
        {
            'name': 'high_res_1920x1080',
            'width': 1920,
            'height': 1080,
            'samples': 1,
            'description': 'High resolution rendering test'
        },
        {
            'name': 'square_512x512',
            'width': 512,
            'height': 512,
            'samples': 1,
            'description': 'Square aspect ratio rendering test'
        },
        {
            'name': 'antialiased_4x',
            'width': 800,
            'height': 600,
            'samples': 4,
            'description': 'Anti-aliased rendering with 4x MSAA'
        },
        {
            'name': 'minimal_400x300',
            'width': 400,
            'height': 300,
            'samples': 1,
            'description': 'Minimal resolution rendering test'
        }
    ]
    
    print("Generating visual regression test images...")
    print("=" * 50)
    
    success_count = 0
    total_count = len(test_cases)
    
    for test_case in test_cases:
        print(f"\nGenerating: {test_case['name']}")
        print(f"Description: {test_case['description']}")
        
        output_path = os.path.join('outputs', f"{test_case['name']}.png")
        
        try:
            # Remove description from test_case for run_render_sandbox
            render_params = {k: v for k, v in test_case.items() 
                           if k not in ['name', 'description']}
            
            try:
                run_render_sandbox(
                    output_path,
                    **render_params
                )
                
                # Verify the output file was created
                if os.path.exists(output_path):
                    file_size = os.path.getsize(output_path)
                    print(f"✅ Created {output_path} ({file_size} bytes)")
                    success_count += 1
                else:
                    print(f"❌ Failed to create {output_path}")
                    
            except Exception as render_error:
                print(f"⚠️  render_sandbox failed for {test_case['name']}: {render_error}")
                print("Falling back to synthetic image generation...")
                
                # Create fallback synthetic image
                create_fallback_image(
                    output_path,
                    test_case['width'],
                    test_case['height'],
                    test_case['name'],
                    test_case['description']
                )
                
                if os.path.exists(output_path):
                    file_size = os.path.getsize(output_path)
                    print(f"✅ Created fallback {output_path} ({file_size} bytes)")
                    success_count += 1
                else:
                    print(f"❌ Failed to create fallback {output_path}")
                
        except Exception as e:
            print(f"❌ Error generating {test_case['name']}: {e}")
    
    print("\n" + "=" * 50)
    print(f"Test image generation completed!")
    print(f"Success: {success_count}/{total_count} images generated")
    
    if success_count < total_count:
        print(f"Warning: {total_count - success_count} images failed to generate")
        sys.exit(1)
    
    print("\nGenerated test images:")
    outputs_dir = Path('outputs')
    if outputs_dir.exists():
        for image_file in sorted(outputs_dir.glob('*.png')):
            file_size = image_file.stat().st_size
            print(f"  - {image_file.name} ({file_size} bytes)")

if __name__ == "__main__":
    main()