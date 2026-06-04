import subprocess
import time
import pyautogui
import pygetwindow as gw
import sys

print("=" * 60)
print("Android Studio Flutter GUI Test Automation")
print("=" * 60)

# Step 1: Check Android Studio is running
print("\n[Step 1] Checking Android Studio...")
windows = gw.getAllWindows()
studio_window = None
for w in windows:
    if 'studio' in w.title.lower() and 'android' in w.title.lower():
        studio_window = w
        print(f"Found: {w.title}")
        break

if not studio_window:
    # Start Android Studio
    print("Starting Android Studio...")
    subprocess.Popen(['C:\\Program Files\\Android\\Android Studio\\bin\\studio64.exe'])
    time.sleep(30)
    windows = gw.getAllWindows()
    for w in windows:
        if 'studio' in w.title.lower():
            studio_window = w
            break

# Step 2: Open Flutter project
print("\n[Step 2] Opening Flutter project...")
studio_window.activate()
time.sleep(1)

# Take screenshot before action
pyautogui.screenshot().save('D:/dingsun/acp-ui/test_step2_before.png')
print("Screenshot: test_step2_before.png")

# Click "Open" button (using keyboard shortcut Ctrl+O)
pyautogui.hotkey('ctrl', 'o')
time.sleep(2)

# Type project path
pyautogui.write('D:\\dingsun\\acp-ui\\acp_ui_flutter')
time.sleep(1)
pyautogui.press('enter')
time.sleep(5)

print("Project path entered")
pyautogui.screenshot().save('D:/dingsun/acp-ui/test_step2_after.png')
print("Screenshot: test_step2_after.png")

# Step 3: Wait for project to load
print("\n[Step 3] Waiting for project to load...")
time.sleep(30)
pyautogui.screenshot().save('D:/dingsun/acp-ui/test_step3_loaded.png')
print("Screenshot: test_step3_loaded.png")

# Step 4: Check for Android emulator
print("\n[Step 4] Checking Android emulator...")
result = subprocess.run(
    ['C:\\Users\\Administrator\\AppData\\Local\\Android\\Sdk\\emulator\\emulator.exe', '-list-avds'],
    capture_output=True, text=True
)
print(f"Available emulators: {result.stdout}")

# Step 5: Start emulator
print("\n[Step 5] Starting emulator...")
if 'test_avd' in result.stdout:
    subprocess.Popen([
        'C:\\Users\\Administrator\\AppData\\Local\\Android\\Sdk\\emulator\\emulator.exe',
        '-avd', 'test_avd'
    ])
    print("Emulator starting...")
    time.sleep(60)  # Wait for emulator to boot

# Step 6: Run Flutter app
print("\n[Step 6] Running Flutter app...")
studio_window.activate()
time.sleep(1)

# Find and click Run button (Shift+F10 or Ctrl+R)
pyautogui.hotkey('shift', 'f10')
time.sleep(2)

pyautogui.screenshot().save('D:/dingsun/acp-ui/test_step6_run.png')
print("Screenshot: test_step6_run.png")

print("\n[Summary] Test screenshots saved:")
print("- test_step2_before.png")
print("- test_step2_after.png")
print("- test_step3_loaded.png")
print("- test_step6_run.png")