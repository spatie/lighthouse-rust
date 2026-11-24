import {readFileSync, writeFileSync} from 'node:fs';

const input = JSON.parse(readFileSync(0, 'utf8'));
let chrome;
let chromeLauncher;
let timedOut = false;
let timer;

try {
    timer = setTimeout(() => {
        timedOut = true;
        // killAll also covers a Chrome launch that has not resolved yet.
        chromeLauncher?.killAll();
        process.exit(124);
    }, input.timeout);

    const {default: lighthouse} = await import('lighthouse');
    chromeLauncher = await import('chrome-launcher');
    chrome = await chromeLauncher.launch(input.chromeOptions);
    const flags = {logLevel: 'error', port: chrome.port};
    if (input.maxWaitForLoad !== null) {
        flags.maxWaitForLoad = input.maxWaitForLoad;
    }
    const result = await lighthouse(input.url, flags, input.lighthouseConfig);
    // Use lhr rather than report[0]: custom output settings may omit JSON or reorder formats.
    const {ReportGenerator} = await import('lighthouse/report/generator/report-generator.js');
    const html = ReportGenerator.generateReport(result.lhr, 'html');
    writeFileSync(1, JSON.stringify({
        report: [result.lhr, html],
        runtimeError: result.lhr.runtimeError ?? null,
    }));
} catch (error) {
    if (!timedOut) {
        console.error(error.stack ?? String(error));
        process.exitCode = 1;
    }
} finally {
    clearTimeout(timer);
    chromeLauncher?.killAll();
}
