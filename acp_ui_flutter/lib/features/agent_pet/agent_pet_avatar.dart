import 'package:flutter/material.dart';
import '../../data/models/agent_realtime/agent_realtime_types.dart';
import '../../data/models/agent_realtime/agent_pet_types.dart';

/// Agent宠物头像 (Phase 3)
/// 类人宠物概念的Agent拟人化
class AgentPetAvatar extends StatefulWidget {
  final String agentId;
  final String agentName;
  final AgentActivityType currentActivity;
  final AgentEmotionType emotion;
  final PetAppearanceType? petType;
  final double size;

  const AgentPetAvatar({
    super.key,
    required this.agentId,
    required this.agentName,
    required this.currentActivity,
    required this.emotion,
    this.petType,
    this.size = 80,
  });

  @override
  State<AgentPetAvatar> createState() => _AgentPetAvatarState();
}

class _AgentPetAvatarState extends State<AgentPetAvatar>
    with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<double> _scaleAnimation;
  PetAnimationType _currentAnimation = PetAnimationType.idle;
  bool _isAnimating = false;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 200),
      vsync: this,
    );
    _scaleAnimation = Tween<double>(begin: 1.0, end: 1.2).animate(
      CurvedAnimation(parent: _animationController, curve: Curves.easeInOut),
    );
    _updateAnimationFromActivity(widget.currentActivity);
  }

  @override
  void didUpdateWidget(AgentPetAvatar oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.currentActivity != widget.currentActivity) {
      _updateAnimationFromActivity(widget.currentActivity);
    }
    if (oldWidget.emotion != widget.emotion) {
      _triggerBlinkAnimation();
    }
  }

  void _updateAnimationFromActivity(AgentActivityType activity) {
    final animationMap = {
      AgentActivityType.thinking: PetAnimationType.think,
      AgentActivityType.executing: PetAnimationType.working,
      AgentActivityType.outputting: PetAnimationType.happy,
      AgentActivityType.waiting: PetAnimationType.idle,
      AgentActivityType.idle: PetAnimationType.blink,
      AgentActivityType.error: PetAnimationType.confused,
    };
    _triggerAnimation(animationMap[activity] ?? PetAnimationType.idle);
  }

  void _triggerAnimation(PetAnimationType animation) {
    setState(() {
      _currentAnimation = animation;
      _isAnimating = true;
    });

    final config = petAnimations[animation]!;
    Future.delayed(Duration(milliseconds: config.duration), () {
      if (_currentAnimation == animation) {
        setState(() {
          _currentAnimation = PetAnimationType.idle;
          _isAnimating = false;
        });
      }
    });
  }

  void _triggerBlinkAnimation() {
    if (!_isAnimating) {
      _triggerAnimation(PetAnimationType.blink);
    }
  }

  void _handlePet() {
    _animationController.forward().then(() => _animationController.reverse());
    _triggerAnimation(PetAnimationType.happy);
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final expression = emotionExpressions[widget.emotion]!;
    final animConfig = petAnimations[_currentAnimation]!;
    final color = _parseColor(expression.color);

    return GestureDetector(
      onTap: _handlePet,
      child: AnimatedBuilder(
        animation: _scaleAnimation,
        builder: (context, child) {
          return Transform.scale(
            scale: _scaleAnimation.value,
            child: Column(
              children: [
                // Pet container
                Container(
                  width: widget.size,
                  height: widget.size,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    color: const Color(0xFF1a1a2e).withOpacity(0.6),
                    border: Border.all(color: color, width: 3),
                  ),
                  child: Stack(
                    alignment: Alignment.center,
                    children: [
                      // Activity glow
                      if (widget.currentActivity != AgentActivityType.idle)
                        Container(
                          width: widget.size,
                          height: widget.size,
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            color: color.withOpacity(0.2),
                          ),
                        ),
                      // Emoji
                      AnimatedSwitcher(
                        duration: const Duration(milliseconds: 300),
                        child: Text(
                          animConfig.expression,
                          key: ValueKey(_currentAnimation),
                          style: TextStyle(fontSize: widget.size * 0.5),
                        ),
                      ),
                      // Level badge
                      Positioned(
                        bottom: -2,
                        right: -2,
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: const Color(0xFF8b5cf6).withOpacity(0.8),
                            borderRadius: BorderRadius.circular(6),
                          ),
                          child: const Text(
                            'Lv.1',
                            style: TextStyle(fontSize: 10, color: Colors.white, fontWeight: FontWeight.w600),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 8),
                // Interaction buttons
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    _buildInteractButton('👋', Colors.green),
                    _buildInteractButton('👉', Colors.orange),
                    _buildInteractButton('🍖', Colors.blue),
                    _buildInteractButton('🎾', Colors.purple),
                  ],
                ),
                const SizedBox(height: 4),
                // Name
                Text(
                  widget.agentName,
                  style: const TextStyle(fontSize: 11, color: Color(0xFF8b8b9b)),
                ),
              ],
            ),
          );
        },
      ),
    );
  }

  Widget _buildInteractButton(String emoji, Color color) {
    return GestureDetector(
      onTap: () => _handlePet(),
      child: Container(
        width: 32,
        height: 32,
        margin: const EdgeInsets.only(right: 4),
        decoration: BoxDecoration(
          color: const Color(0xFF1a1a2e).withOpacity(0.6),
          border: Border.all(color: const Color(0xFF3a3a5a)),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Center(child: Text(emoji, style: const TextStyle(fontSize: 16))),
      ),
    );
  }

  Color _parseColor(String hex) {
    return Color(int.parse(hex.replaceFirst('#', '0xFF')));
  }
}