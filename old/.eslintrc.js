module.exports = {
  env: {
    browser: true,
    commonjs: true,
    es2021: true,
  },
  extends: 'airbnb-base',
  parserOptions: {
    ecmaVersion: 12,
  },
  rules: {
    'no-use-before-define': 0,
    'no-console': 0,
  },
};
