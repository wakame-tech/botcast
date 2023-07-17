const voicevox = `http://127.0.0.1:50021`;
const dicts = {
  里志: "サトシ",
};
for await (const [surface, pron] of Object.entries(dicts)) {
  const res = await fetch(
    `${voicevox}/user_dict_word?surface=${encodeURI(
      surface
    )}&pronunciation=${encodeURI(pron)}&accent_type=0&word_type=PROPER_NOUN`,
    {
      method: "POST",
    }
  ).then((res) => res.text());
  console.log(res);
}

console.log(await fetch(`${voicevox}/user_dict`).then((res) => res.json()));
