import subprocess
import time
import pyautogui
import pygetwindow as gw

print("=" * 60)
print("Flutter Android Test - Final Phase")
print("=" * 60)

# Step 1: Create x86_64 emulator
print("\n[Step 1] Creating x86_64 emulator...")
avd_manager = 'D:\\Android\\Sdk\\cmdline-tools\\latest\\bin\\avdmanager.bat'

# Check existing AVDs
result = subprocess.run([avd_manager, 'list', 'avd'], capture_output=True, text=True)
print(f"Existing AVDs: {result.stdout}")

# Create new x86_64 AVD
create_cmd = [
    avd_manager, 'create', 'avd',
    '-n', 'flutter_x86_64',
    '-k', 'system-images;android-34;google_apis;x86_64',
    '-d', 'pixel_6'
]

result = subprocess.run(create_cmd, capture_output=True, text=True, input='no\n', timeout=120)
print(f"Create result: {result.stdout[:500]}")
print(f"Errors: {result.stderr[:500]}")

# Step 2: Start emulator
print("\n[Step 2] Starting x86_64 emulator...")
emulator = 'D:\\Android\\Sdk\\emulator\\emulator.exe'
subprocess.Popen([emulator, '-avd', 'flutter_x86_64'], creationflags=subprocess.CREATE_NEW_CONSOLE)
print("Emulator launching...")

# Wait for emulator to boot
print("Waiting for emulator to boot (60 seconds)...")
time.sleep(60)

# Check if emulator is running
result = subprocess.run(['D:\\Android\\Sdk\\platform-tools\\adb.exe', 'devices'], capture_output=True, text=True)
print(f"ADB devices: {result.stdout}")

# Step 3: Focus on Android Studio and run Flutter
print("\n[Step 3] Operating Android Studio...")
windows = gw.getAllWindows()
for w in windows:
    if 'studio' in w.title.lower():
        print(f"Found: {w.title}")
        w.activate()
        time.sleep(2)

        # Open Flutter project (already should be loaded)
        pyautogui.screenshot().save('D:/dingsun/acp-ui/flutter_before_run.png')

        # Run Flutter app - use Shift+F10 (Run)
        pyautogui.hotkey('shift', 'f10')
        time.sleep(5)

        pyautogui.screenshot().save('D:/dingsun/acp-ui/flutter_after_run.png')

        break

# Step 4: Wait and capture app running
print("\n[Step 4] Waiting for Flutter app to launch...")
time.sleep(30)
pyautogui.screenshot().save('D:/dingsun/acp-ui/flutter_app_running.png')

# Check for emulator window
windows = gw.getAllWindows()
for w in windows:
    if 'emulator' in w.title.lower() or 'pixel' in w.title.lower() or 'android' in w.title.lower():
        print(f"Emulator window: {w.title}")

print("\n[Complete] Screenshots saved:")
print("- flutter_before_run.png")
print("- flutter_after_run.png")
print("- flutter_app_running.png")