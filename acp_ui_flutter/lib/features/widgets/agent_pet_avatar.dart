/// Agent Pet Avatar
/// Agent宠物头像组件

import 'package:flutter/material.dart';
import '../../data/models/agent_pet/agent_pet_types.dart';

class AgentPetAvatar extends StatefulWidget {
  final AgentPet pet;
  final double size;
  final bool showAnimation;
  final VoidCallback? onTap;

  const AgentPetAvatar({
    super.key,
    required this.pet,
    this.size = 48,
    this.showAnimation = false,
    this.onTap,
  });

  @override
  State<AgentPetAvatar> createState() => _AgentPetAvatarState();
}

class _AgentPetAvatarState extends State<AgentPetAvatar>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _scaleAnimation;
  late Animation<double> _rotationAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 1000),
      vsync: this,
    );

    _scaleAnimation = Tween<double>(begin: 1.0, end: 1.1).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeInOut),
    );

    _rotationAnimation = Tween<double>(begin: 0, end: 0.05).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeInOut),
    );

    if (widget.showAnimation && _shouldAnimate()) {
      _controller.repeat(reverse: true);
    }
  }

  bool _shouldAnimate() {
    switch (widget.pet.currentAnimation) {
      case PetAnimation.idle:
        return false;
      case PetAnimation.walking:
      case PetAnimation.thinking:
      case PetAnimation.happy:
      case PetAnimation.sleeping:
      case PetAnimation.eating:
      case PetAnimation.playing:
        return true;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: widget.onTap,
      child: AnimatedBuilder(
        animation: _controller,
        builder: (context, child) {
          return Transform.scale(
            scale: _scaleAnimation.value,
            child: Transform.rotate(
              angle: _rotationAnimation.value,
              child: child,
            ),
          );
        },
        child: Container(
          width: widget.size,
          height: widget.size,
          decoration: BoxDecoration(
            shape: BoxShape.circle,
            color: _getBackgroundColor(),
            border: Border.all(
              color: _getBorderColor(),
              width: 2,
            ),
            boxShadow: [
              BoxShadow(
                color: _getBorderColor().withOpacity(0.3),
                blurRadius: 8,
                spreadRadius: 2,
              ),
            ],
          ),
          child: Stack(
            alignment: Alignment.center,
            children: [
              // 主体表情
              Text(
                _getEmotionEmoji(),
                style: TextStyle(fontSize: widget.size * 0.5),
              ),
              // 状态图标（右上角）
              Positioned(
                top: 0,
                right: 0,
                child: _buildStatusIndicator(),
              ),
              // Lv徽章（底部）
              if (widget.size >= 40)
                Positioned(
                  bottom: 0,
                  child: _buildLevelBadge(),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Color _getBackgroundColor() {
    switch (widget.pet.emotion) {
      case PetEmotion.happy:
        return Colors.green.withOpacity(0.1);
      case PetEmotion.thinking:
        return Colors.blue.withOpacity(0.1);
      case PetEmotion.sleepy:
        return Colors.grey.withOpacity(0.1);
      case PetEmotion.confused:
        return Colors.orange.withOpacity(0.1);
      case PetEmotion.excited:
        return Colors.purple.withOpacity(0.1);
      case PetEmotion.bored:
        return Colors.yellow.withOpacity(0.1);
      case PetEmotion.calm:
        return Colors.teal.withOpacity(0.1);
    }
  }

  Color _getBorderColor() {
    switch (widget.pet.emotion) {
      case PetEmotion.happy:
        return Colors.green;
      case PetEmotion.thinking:
        return Colors.blue;
      case PetEmotion.sleepy:
        return Colors.grey;
      case PetEmotion.confused:
        return Colors.orange;
      case PetEmotion.excited:
        return Colors.purple;
      case PetEmotion.bored:
        return Colors.yellow;
      case PetEmotion.calm:
        return Colors.teal;
    }
  }

  String _getEmotionEmoji() {
    switch (widget.pet.emotion) {
      case PetEmotion.happy:
        return '😊';
      case PetEmotion.thinking:
        return '🤔';
      case PetEmotion.sleepy:
        return '😴';
      case PetEmotion.confused:
        return '😕';
      case PetEmotion.excited:
        return '🎉';
      case PetEmotion.bored:
        return '😐';
      case PetEmotion.calm:
        return '🙂';
    }
  }

  Widget _buildStatusIndicator() {
    IconData icon;
    Color color;

    switch (widget.pet.emotion) {
      case PetEmotion.happy:
        icon = Icons.check_circle;
        color = Colors.green;
        break;
      case PetEmotion.thinking:
        icon = Icons.psychology;
        color = Colors.blue;
        break;
      case PetEmotion.sleepy:
        icon = Icons.bedtime;
        color = Colors.grey;
        break;
      case PetEmotion.confused:
        icon = Icons.error;
        color = Colors.orange;
        break;
      case PetEmotion.excited:
        icon = Icons.bolt;
        color = Colors.purple;
        break;
      case PetEmotion.bored:
        icon = Icons.hourglass_empty;
        color = Colors.yellow;
        break;
      case PetEmotion.calm:
        icon = Icons.circle;
        color = Colors.teal;
        break;
    }

    return Container(
      width: widget.size * 0.25,
      height: widget.size * 0.25,
      decoration: BoxDecoration(
        color: color,
        shape: BoxShape.circle,
      ),
      child: Icon(icon, size: widget.size * 0.15, color: Colors.white),
    );
  }

  Widget _buildLevelBadge() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
      decoration: BoxDecoration(
        color: Colors.purple,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        'Lv.${widget.pet.level}',
        style: const TextStyle(
          fontSize: 8,
          color: Colors.white,
          fontWeight: FontWeight.bold,
        ),
      ),
    );
  }
}