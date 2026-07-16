// WebdriverIO 配置 - Tauri E2E 测试
exports.config = {
    runner: 'local',
    specs: [
        './test/**/*.test.cjs'
    ],
    maxInstances: 1,
    capabilities: [{
        browserName: 'tauri',
        'tauri:options': {
            application: './src-tauri/target/debug/acp-ui.exe'
        }
    }],
    logLevel: 'info',
    bail: 0,
    baseUrl: 'http://localhost',
    waitforTimeout: 10000,
    connectionRetryTimeout: 120000,
    connectionRetryCount: 3,
    services: ['tauri'],
    framework: 'mocha',
    reporters: ['spec'],
    mochaOpts: {
        ui: 'bdd',
        timeout: 60000
    }
}
