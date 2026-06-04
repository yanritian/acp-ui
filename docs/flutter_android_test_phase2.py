import subprocess
import time
import pyautogui
import pygetwindow as gw

print("=" * 60)
print("Flutter Android Test - Phase 2")
print("=" * 60)

# Step 1: List available system images
print("\n[Step 1] Checking available system images...")
sdk_manager = 'C:\\Users\\Administrator\\AppData\\Local\\Android\\Sdk\\cmdline-tools\\latest\\bin\\sdkmanager.bat'
result = subprocess.run([sdk_manager, '--list'], capture_output=True, text=True, timeout=60)
print(result.stdout[:2000] if result.stdout else "No output")

# Step 2: Create x86 emulator
print("\n[Step 2] Creating x86 emulator...")
avd_manager = 'C:\\Users\\Administrator\\AppData\\Local\\Android\\Sdk\\cmdline-tools\\latest\\bin\\avdmanager.bat'

# Create new AVD with x86
create_cmd = [
    avd_manager, 'create', 'avd',
    '-n', 'flutter_test_x86',
    '-k', 'system-images;android-34;google_apis;x86',
    '-d', 'pixel_6'
]

result = subprocess.run(create_cmd, capture_output=True, text=True, input='no\n', timeout=120)
print(f"Create result: {result.stdout}")
print(f"Errors: {result.stderr}")

# Step 3: Find Android Studio window and interact
print("\n[Step 3] Operating Android Studio...")
windows = gw.getAllWindows()
for w in windows:
    print(f"Window: {w.title}")

# Find Studio window
studio = None
for w in windows:
    if 'studio' in w.title.lower():
        studio = w
        break

if studio:
    print(f"Activating: {studio.title}")
    studio.activate()
    time.sleep(2)

    # Take screenshot of current state
    pyautogui.screenshot().save('D:/dingsun/acp-ui/android_studio_state.png')
    print("Screenshot saved: android_studio_state.png")

    # Click on the project area to focus
    # Use Ctrl+Shift+N to open file
    pyautogui.hotkey('ctrl', 'shift', 'n')
    time.sleep(1)

    # Type main.dart to open Flutter main file
    pyautogui.write('main.dart')
    time.sleep(1)
    pyautogui.press('enter')
    time.sleep(3)

    pyautogui.screenshot().save('D:/dingsun/acp-ui/flutter_main_opened.png')
    print("Screenshot saved: flutter_main_opened.png")

# Step 4: Check device manager
print("\n[Step 4] Opening Device Manager...")
if studio:
    studio.activate()
    time.sleep(1)

    # Try to open Device Manager via menu
    # Alt+T for Tools menu, then D for Device Manager
    pyautogui.hotkey('alt', 't')
    time.sleep(1)
    pyautogui.press('d')
    time.sleep(3)

    pyautogui.screenshot().save('D:/dingsun/acp-ui/device_manager.png')
    print("Screenshot saved: device_manager.png")

print("\n[Complete] All screenshots saved to D:/dingsun/acp-ui/")