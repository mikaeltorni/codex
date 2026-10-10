// Run with node; type-check with tsc --allowJs --checkJs --noEmit --strict --target ES2020.

/**
 * @param {import('../schema/typescript/v2/UsageLimitWaitChangedNotification.js').UsageLimitWaitChangedNotification} notification
 * @param {number} now
 */
function remainingMilliseconds(notification, now) {
  return notification.retryAtMs === null
    ? null
    : Math.max(0, notification.retryAtMs - now);
}

const actual = [
  '{"threadId":"waiting","retryAtMs":15000}',
  '{"threadId":"expired","retryAtMs":9999}',
  '{"threadId":"ended","retryAtMs":null}',
].map((wire) => remainingMilliseconds(JSON.parse(wire), 10000));

if (JSON.stringify(actual) !== JSON.stringify([5000, 0, null])) {
  throw new Error(`Unexpected countdowns: ${JSON.stringify(actual)}`);
}
