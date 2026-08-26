const fs = require('fs');

const review = JSON.parse(fs.readFileSync('translations_review.json', 'utf8'));

// App paths
const appRuPath = 'apps/app-frontend/src/locales/ru-RU/index.json';
const appRu = JSON.parse(fs.readFileSync(appRuPath, 'utf8'));
const appEnPath = 'apps/app-frontend/src/locales/en-US/index.json';
const appEn = JSON.parse(fs.readFileSync(appEnPath, 'utf8'));

// UI paths
const uiRuPath = 'packages/ui/src/locales/ru-RU/index.json';
const uiRu = JSON.parse(fs.readFileSync(uiRuPath, 'utf8'));
const uiEnPath = 'packages/ui/src/locales/en-US/index.json';
const uiEn = JSON.parse(fs.readFileSync(uiEnPath, 'utf8'));

let updatedAppRu = 0;
let updatedAppEn = 0;
let updatedUiRu = 0;
let updatedUiEn = 0;

for (const [key, item] of Object.entries(review)) {
	// app-frontend
	if (item.package === 'app-frontend' || item.package === 'both') {
		if (item.ru !== undefined) {
			if (appRu[key]) {
				if (typeof appRu[key] === 'string') {
					appRu[key] = item.ru;
				} else {
					appRu[key].message = item.ru;
					appRu[key].defaultMessage = item.ru;
				}
			} else {
				appRu[key] = { message: item.ru, defaultMessage: item.ru };
			}
			updatedAppRu++;
		}

		if (item.en !== undefined) {
			if (appEn[key]) {
				if (typeof appEn[key] === 'string') {
					appEn[key] = item.en;
				} else {
					appEn[key].message = item.en;
					appEn[key].defaultMessage = item.en;
				}
			} else {
				appEn[key] = { message: item.en, defaultMessage: item.en };
			}
			updatedAppEn++;
		}
	}

	// UI
	if (item.package === 'ui' || item.package === 'both') {
		if (item.ru !== undefined) {
			if (uiRu[key]) {
				if (typeof uiRu[key] === 'string') {
					uiRu[key] = item.ru;
				} else {
					uiRu[key].message = item.ru;
					uiRu[key].defaultMessage = item.ru;
				}
			} else {
				uiRu[key] = { message: item.ru, defaultMessage: item.ru };
			}
			updatedUiRu++;
		}

		if (item.en !== undefined) {
			if (uiEn[key]) {
				if (typeof uiEn[key] === 'string') {
					uiEn[key] = item.en;
				} else {
					uiEn[key].message = item.en;
					uiEn[key].defaultMessage = item.en;
				}
			} else {
				uiEn[key] = { message: item.en, defaultMessage: item.en };
			}
			updatedUiEn++;
		}
	}
}

fs.writeFileSync(appRuPath, JSON.stringify(appRu, null, 2), 'utf8');
fs.writeFileSync(appEnPath, JSON.stringify(appEn, null, 2), 'utf8');
fs.writeFileSync(uiRuPath, JSON.stringify(uiRu, null, 2), 'utf8');
fs.writeFileSync(uiEnPath, JSON.stringify(uiEn, null, 2), 'utf8');

console.log(`Successfully applied all translations (RU and EN)!
App-Frontend: ${updatedAppRu} RU / ${updatedAppEn} EN
UI: ${updatedUiRu} RU / ${updatedUiEn} EN`);
