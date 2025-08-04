#!/bin/bash
# cleanup_examples.sh
# Script untuk organize dan cleanup example files

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🧹 Cleaning up redundant example files...${NC}"
echo "=============================================="

# Check if we're in the right directory
if [ ! -d "examples" ]; then
    echo -e "${RED}❌ Error: examples directory not found${NC}"
    echo -e "${YELLOW}💡 Run this script from whisper-rust-binding root directory${NC}"
    exit 1
fi

# Backup current examples
echo -e "${YELLOW}📋 Creating backup...${NC}"
if [ -d "examples_backup" ]; then
    rm -rf examples_backup
fi
cp -r examples examples_backup
echo -e "${GREEN}✅ Backup created at examples_backup/${NC}"

# Create organized directory structure
echo -e "${BLUE}📁 Creating organized structure...${NC}"
mkdir -p examples/01_basic
mkdir -p examples/02_production
mkdir -p examples/03_flutter
mkdir -p examples/04_advanced

# Move useful files to organized structure
echo -e "${BLUE}📦 Moving useful examples...${NC}"

# Basic examples
if [ -f "examples/simple_test.rs" ]; then
    mv examples/simple_test.rs examples/01_basic/
    echo -e "  ${GREEN}✅ Moved simple_test.rs → 01_basic/${NC}"
fi

if [ -f "examples/test_model_only.rs" ]; then
    mv examples/test_model_only.rs examples/01_basic/
    echo -e "  ${GREEN}✅ Moved test_model_only.rs → 01_basic/${NC}"
fi

if [ -f "examples/test_transcription.rs" ]; then
    mv examples/test_transcription.rs examples/01_basic/
    echo -e "  ${GREEN}✅ Moved test_transcription.rs → 01_basic/${NC}"
fi

# Production examples
if [ -f "examples/transcribe_file.rs" ]; then
    mv examples/transcribe_file.rs examples/02_production/
    echo -e "  ${GREEN}✅ Moved transcribe_file.rs → 02_production/ (MAIN EXAMPLE)${NC}"
fi

if [ -f "examples/production_test.rs" ]; then
    mv examples/production_test.rs examples/02_production/
    echo -e "  ${GREEN}✅ Moved production_test.rs → 02_production/${NC}"
fi

if [ -f "examples/test_direct_whisper.rs" ]; then
    mv examples/test_direct_whisper.rs examples/02_production/
    echo -e "  ${GREEN}✅ Moved test_direct_whisper.rs → 02_production/${NC}"
fi

# Flutter examples
if [ -f "examples/flutter_api_demo.rs" ]; then
    mv examples/flutter_api_demo.rs examples/03_flutter/
    echo -e "  ${GREEN}✅ Moved flutter_api_demo.rs → 03_flutter/${NC}"
fi

if [ -f "examples/dual_project_integration.rs" ]; then
    mv examples/dual_project_integration.rs examples/03_flutter/
    echo -e "  ${GREEN}✅ Moved dual_project_integration.rs → 03_flutter/${NC}"
fi

# Advanced examples
if [ -f "examples/simple_sliding_window.rs" ]; then
    mv examples/simple_sliding_window.rs examples/04_advanced/
    echo -e "  ${GREEN}✅ Moved simple_sliding_window.rs → 04_advanced/${NC}"
fi

if [ -f "examples/hybrid_sliding_window.rs" ]; then
    mv examples/hybrid_sliding_window.rs examples/04_advanced/
    echo -e "  ${GREEN}✅ Moved hybrid_sliding_window.rs → 04_advanced/${NC}"
fi

# Move common directory if exists
if [ -d "examples/common" ]; then
    mv examples/common examples/00_common
    echo -e "  ${GREEN}✅ Moved common/ → 00_common/${NC}"
fi

# Delete redundant files
echo -e "${RED}🗑️ Deleting redundant examples...${NC}"

redundant_files=(
    "flutter_api_mock.rs"
    "simple_integration_test.rs"
    "sliding_window.rs"
    "sliding_window_transcribe.rs"
    "realtime_sliding_window.rs"
    "murajaah_chunks.rs"
    "realtime_murajaah.rs"
    "flutter_realtime_demo.rs"
)

for file in "${redundant_files[@]}"; do
    if [ -f "examples/$file" ]; then
        rm "examples/$file"
        echo -e "  ${RED}🗑️ Deleted $file${NC}"
    fi
done

# Clean up any remaining loose files (except directories)
echo -e "${BLUE}🧽 Cleaning up remaining loose files...${NC}"
find examples/ -maxdepth 1 -type f -name "*.rs" -delete

echo ""
echo -e "${GREEN}✅ Cleanup completed!${NC}"
echo ""

# Show new structure
echo -e "${BLUE}📂 New organized structure:${NC}"
if command -v tree >/dev/null 2>&1; then
    tree examples/
else
    find examples/ -type f | sort
fi

echo ""
echo -e "${YELLOW}📋 Summary of kept examples:${NC}"
echo -e "${BLUE}01_basic/${NC} (Learning & Basic Testing)"
echo -e "  • simple_test.rs       - Import verification"
echo -e "  • test_model_only.rs   - Model loading test"
echo -e "  • test_transcription.rs - Basic transcription"
echo ""
echo -e "${BLUE}02_production/${NC} (Real Usage)"
echo -e "  • transcribe_file.rs   - ⭐ MAIN EXAMPLE - File transcription"
echo -e "  • production_test.rs   - Production-ready test"
echo -e "  • test_direct_whisper.rs - Direct whisper.cpp integration"
echo ""
echo -e "${BLUE}03_flutter/${NC} (Flutter Integration)"
echo -e "  • flutter_api_demo.rs  - Flutter API simulation"
echo -e "  • dual_project_integration.rs - Standalone project concept"
echo ""
echo -e "${BLUE}04_advanced/${NC} (Advanced Features)"
echo -e "  • simple_sliding_window.rs - Basic sliding window"
echo -e "  • hybrid_sliding_window.rs - Advanced sliding window"
echo ""

# Test main example
echo -e "${YELLOW}🔧 Testing main example compilation...${NC}"
if cargo check --example transcribe_file >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Main example (transcribe_file) compiles successfully!${NC}"
else
    echo -e "${RED}❌ Main example compilation failed${NC}"
    echo -e "${YELLOW}💡 Run: cargo check --example transcribe_file${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Examples cleanup and organization completed!${NC}"
echo -e "${YELLOW}💡 Next steps:${NC}"
echo -e "   1. Test main example: cargo run --example transcribe_file ggml-tiny.bin audio.wav"
echo -e "   2. Update Cargo.toml example paths if needed"
echo -e "   3. Update documentation with new structure"
