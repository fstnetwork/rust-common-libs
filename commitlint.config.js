module.exports = {
  extends: [
    process.env.IN_NIX_SHELL ? process.env.COMMITLINT_PRESET : "@commitlint/config-conventional",
  ],
  rules: {
    "body-max-line-length": [2, "always", 120],
  },
};
