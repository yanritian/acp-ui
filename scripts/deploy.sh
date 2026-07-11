#!/bin/bash

# Hermes Game Operator 部署脚本
# 用法: ./scripts/deploy.sh [环境]

set -e

ENVIRONMENT=${1:-production}
PROJECT_NAME="hermes-game-operator"
VERSION=${VERSION:-"0.1.0-alpha"}

echo "🚀 开始部署 Hermes Game Operator"
echo "环境: $ENVIRONMENT"
echo "版本: $VERSION"

# 检查依赖
check_dependencies() {
    echo "📋 检查依赖..."

    if ! command -v docker &> /dev/null; then
        echo "❌ Docker 未安装"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        echo "❌ Docker Compose 未安装"
        exit 1
    fi

    echo "✅ 依赖检查通过"
}

# 构建镜像
build() {
    echo "🔨 构建镜像..."

    docker-compose build

    echo "✅ 构建完成"
}

# 运行测试
test() {
    echo "🧪 运行测试..."

    # 前端测试
    docker-compose run --rm frontend npm run test

    # 后端测试
    docker-compose run --rm backend cargo test

    echo "✅ 测试通过"
}

# 部署到生产环境
deploy_production() {
    echo "🚀 部署到生产环境..."

    # 拉取最新代码
    git pull origin main

    # 构建
    docker-compose build

    # 滚动更新
    docker-compose up -d --remove-orphans

    # 等待服务启动
    sleep 10

    # 健康检查
    if curl -f http://localhost:80/health; then
        echo "✅ 部署成功"
    else
        echo "❌ 部署失败"
        exit 1
    fi
}

# 部署到测试环境
deploy_staging() {
    echo "🧪 部署到测试环境..."

    docker-compose -f docker-compose.yml -f docker-compose.staging.yml up -d --build

    echo "✅ 测试环境部署完成"
}

# 回滚
rollback() {
    echo "⏮️  回滚到上一个版本..."

    docker-compose down
    docker-compose up -d

    echo "✅ 回滚完成"
}

# 清理
cleanup() {
    echo "🧹 清理资源..."

    docker system prune -f
    docker volume prune -f

    echo "✅ 清理完成"
}

# 主函数
main() {
    check_dependencies

    case $ENVIRONMENT in
        production)
            test
            deploy_production
            ;;
        staging)
            test
            deploy_staging
            ;;
        rollback)
            rollback
            ;;
        cleanup)
            cleanup
            ;;
        *)
            echo "❌ 未知环境: $ENVIRONMENT"
            echo "用法: ./scripts/deploy.sh [production|staging|rollback|cleanup]"
            exit 1
            ;;
    esac

    echo "🎉 部署完成！"
}

main