#!/usr/bin/env python3
"""
Test script that generates visual regression test images using render_sandbox.
This script creates various rendering scenarios to test visual output consistency.
Requires a real GPU - CI will fail if render_sandbox cannot run properly.
"""

import os
import subprocess
import sys
import tempfile
from pathlib import Path

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
                
        except Exception as e:
            print(f"❌ Error generating {test_case['name']}: {e}")
            # Don't continue with synthetic fallback - fail the test
            sys.exit(1)
    
    print("\n" + "=" * 50)
    print(f"Test image generation completed!")
    print(f"Success: {success_count}/{total_count} images generated")
    
    if success_count < total_count:
        print(f"Error: {total_count - success_count} images failed to generate")
        print("Visual regression testing requires GPU access - CI should fail without real GPU")
        sys.exit(1)
    
    print("\nGenerated test images:")
    outputs_dir = Path('outputs')
    if outputs_dir.exists():
        for image_file in sorted(outputs_dir.glob('*.png')):
            file_size = image_file.stat().st_size
            print(f"  - {image_file.name} ({file_size} bytes)")

if __name__ == "__main__":
    main()