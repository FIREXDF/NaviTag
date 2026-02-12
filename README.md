<p align="center">
  <img src="src/logo.png" alt="NaviTag Logo" width="150">
</p>


<h1 align="center">NaviTag</h1>

A modern, cross-platform GUI application for organizing and tagging your music library. Built with Rust and Iced, NaviTag allows you to easily edit metadata, fetch information from online sources (Apple Music, Spotify, Genius, Last.fm), and download high-quality cover art.

## Features

-   **File Browser**: Open any directory to list its music files.
-   **Metadata Search**: Select a file, search for it online (Title, Artist, Album), and apply the results.
-   **Batch Tagging**: Automatically search for and apply metadata to all files in the folder at once.
-   **Online Search Integration**:
    -   **Apple Music** (Enabled by default)
    -   **Spotify** (Requires Client ID/Secret) 
    -   **Genius** (Requires Access Token)
    -   **Last.fm** (Requires API Key)
-   **Cover Art Downloading**: Fetch high-resolution artwork from online sources.
-   **Auto-Save**: Changes are automatically saved after a short delay, or manually via "Save All".
-   **Dark Mode UI**: Clean and intuitive interface designed for efficiency.

## Prerequisites

-   **Rust & Cargo**: To build from source, you need the Rust toolchain installed. Get it from [rustup.rs](https://rustup.rs).

## Installation & Building

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/yourusername/NaviTag.git
    cd NaviTag
    ```

2.  **Run the application**:
    ```bash
    cargo run
    ```
    *Note: The first run may take a few minutes to compile dependencies.*

3.  **Build a release executable**:
    For a standalone optimized binary:
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/navitag.exe` (Windows) or `target/release/navitag` (Linux/macOS).

## Usage Guide

1.  **Open a Folder**: Click "Open Folder" to select a directory containing your music files.
2.  **Select a File**: Click on any file in the left panel to load its details into the editor.
3.  **Edit Metadata**:
    -   Select a file.
    -   Use the **Online Search** (right panel) to find metadata for that specific track.
    -   Click **Apply** on a result to update the file.
4.  **Batch Tagging**:
    -   Click **Batch Tag (Folder)** to automatically search for and apply metadata to *all* files in the folder at once.
5.  **Save Changes**:
    -   Changes are auto-saved briefly after editing.
    -   Click **Save All** to force save all changes immediately.

## Configuration

NaviTag supports multiple metadata providers. You can configure them in the **Settings** menu:

1.  Click the **Settings** button on the main screen or in the search panel.
2.  **Apple Music**: Enabled by default (no key required).
3.  **Spotify**:
    -   Enable the checkbox.
    -   Enter your **Client ID** and **Client Secret** (from [Spotify Developer Dashboard](https://developer.spotify.com/dashboard/)).
4.  **Genius**:
    -   Enable the checkbox.
    -   Enter your **Access Token** (from [Genius API Client Management](https://genius.com/api-clients)).
5.  **Last.fm**:
    -   Enable the checkbox.
    -   Enter your **API Key** (from [Last.fm API Account](https://www.last.fm/api/account/create)).
6.  Click **Save & Close** to persist your settings.
