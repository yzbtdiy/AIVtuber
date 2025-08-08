# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AIVtuber is a Bilibili virtual livestreamer application built with Tauri + Vue.js + Three.js VRM + Rust. The app connects to Bilibili's livestream API to receive real-time messages (danmu, gifts, etc.), processes them with AI for content generation, converts responses to speech using TTS, and displays a VRM virtual character with expressions and animations.

## Development Commands

### Core Commands
- `bun install` - Install dependencies
- `bun run dev` - Start development server (frontend only)
- `bun run tauri dev` - Start full Tauri development mode (recommended)
- `bun run build` - Build frontend for production
- `bun run tauri build` - Build complete desktop application

### Type Checking
- `vue-tsc --noEmit` - Run TypeScript type checking (part of build process)

## Architecture Overview

### Frontend Architecture (Vue.js + TypeScript)
- **Composables-based**: Heavy use of Vue 3 composition API with dedicated composables for different concerns
- **Component Structure**: Single main view (`LiveRoom.vue`) orchestrates multiple specialized components
- **Key Composables**:
  - `useBilibiliConnection` - Manages connection state and API calls to Rust backend
  - `useMessageHandler` - Processes incoming messages and manages message history
  - `useDanmuConfig` - Handles danmu display configuration
  - `useBilibiliEventListener` - Event listening logic for real-time updates
  - `useVTuberManager` - Three.js VRM character management

### Backend Architecture (Rust + Tauri)
- **Modular Design**: Three main modules in `src-tauri/src/`:
  - `api/` - Tauri command handlers exposed to frontend
  - `core/` - Core data structures, protocol parsing, and state management  
  - `services/` - External service integrations (Bilibili, OpenAI, TTS)
- **State Management**: Uses Tauri's managed state with `ClientState` and `ProxyState`
- **Key Features**:
  - WebSocket client for Bilibili livestream protocol
  - HTTP proxy server for development
  - Protocol parsing for Bilibili's binary message format
  - OpenAI chat integration
  - TTS (Text-to-Speech) service integration

### Configuration System
- Uses `config.json` file (see `config.example.json` for structure)
- Requires Bilibili Open Platform credentials: `id_code`, `app_id`, `access_key`, `access_secret`
- Optional OpenAI and TTS service configuration
- Configuration loaded/saved through Tauri API calls

### VRM Integration
- Three.js-based VRM character rendering in `VTuberCanvas` component
- Uses `@pixiv/three-vrm` and `@pixiv/three-vrm-animation` libraries
- Character models stored in `/public/` directory (`.vrm`, `.vrma` files)
- Audio-driven lip sync and emotion-based animation system

## Key Technical Details

### Message Flow
1. Frontend calls `connectBilibili()` with config
2. Rust backend establishes WebSocket connection to Bilibili
3. Binary messages parsed using custom protocol implementation
4. Messages forwarded to frontend via Tauri events
5. Frontend processes messages through composables
6. AI responses generated via OpenAI integration
7. TTS converts responses to audio
8. VRM character plays audio with lip sync

### Bilibili Protocol
- Custom binary protocol implementation in `src-tauri/src/core/proto.rs`
- Supports multiple message types: danmu, gifts, super chat, subscriptions, likes, etc.
- Authentication using Bilibili Open Platform credentials
- Automatic reconnection on connection failure

### Development Patterns
- Frontend uses TypeScript with strict mode enabled
- Rust backend follows standard module organization with comprehensive documentation
- Event-driven communication between frontend and backend via Tauri
- Composable pattern for Vue.js logic separation and reusability