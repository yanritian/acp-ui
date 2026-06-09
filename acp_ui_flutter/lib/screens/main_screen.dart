import 'package:flutter/material.dart';
import 'approval_screen.dart' show ApprovalStateNotifier;
import 'progress_screen.dart';
import 'approval_screen.dart';
import 'instruction_screen.dart';

/// Main mobile shell with bottom navigation — 3 tabs:
/// 1. Progress (看进度)
/// 2. Approvals (做审批)
/// 3. Instructions (发指令)
class MainScreen extends StatefulWidget {
  const MainScreen({super.key});

  @override
  State<MainScreen> createState() => _MainScreenState();
}

class _MainScreenState extends State<MainScreen> {
  int _currentIndex = 0;
  int _pendingApprovalCount = 0;

  final List<Widget> _screens = const [
    ProgressScreen(),
    ApprovalScreen(),
    InstructionScreen(),
  ];

  void _updateApprovalCount() {
    setState(() {
      _pendingApprovalCount = ApprovalStateNotifier.pendingApprovals;
    });
  }

  @override
  void initState() {
    super.initState();
    ApprovalStateNotifier.addListener(_updateApprovalCount);
  }

  @override
  void dispose() {
    ApprovalStateNotifier.removeListener(_updateApprovalCount);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return Scaffold(
      body: IndexedStack(
        index: _currentIndex,
        children: _screens,
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (index) => setState(() => _currentIndex = index),
        labelBehavior: NavigationDestinationLabelBehavior.alwaysShow,
        indicatorColor: colorScheme.primaryContainer,
        destinations: [
          const NavigationDestination(
            icon: Icon(Icons.track_changes_outlined),
            selectedIcon: Icon(Icons.track_changes),
            label: 'Progress',
          ),
          NavigationDestination(
            icon: Stack(
              clipBehavior: Clip.none,
              children: [
                const Icon(Icons.notifications_none),
                Positioned(
                  right: -4,
                  top: -4,
                  child: Container(
                    padding: const EdgeInsets.all(2),
                    decoration: const BoxDecoration(
                      color: Colors.red,
                      shape: BoxShape.circle,
                    ),
                    constraints: const BoxConstraints(
                      minWidth: 14,
                      minHeight: 14,
                    ),
                    child: Text(
                      '$_pendingApprovalCount',
                      style: TextStyle(color: Colors.white, fontSize: 8, fontWeight: FontWeight.bold),
                      textAlign: TextAlign.center,
                    ),
                  ),
                ),
              ],
            ),
            selectedIcon: const Icon(Icons.notifications),
            label: 'Approvals',
          ),
          const NavigationDestination(
            icon: Icon(Icons.send_outlined),
            selectedIcon: Icon(Icons.send),
            label: 'Instructions',
          ),
        ],
      ),
    );
  }
}
