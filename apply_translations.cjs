const fs = require('fs');

const review = JSON.parse(fs.readFileSync('translations_review.json', 'utf8'));

const appRuPath = 'apps/app-frontend/src/locales/ru-RU/index.json';
const appRu = JSON.parse(fs.readFileSync(appRuPath, 'utf8'));

const uiRuPath = 'packages/ui/src/locales/ru-RU/index.json';
const uiRu = JSON.parse(fs.readFileSync(uiRuPath, 'utf8'));

let updatedApp = 0;
let updatedUi = 0;

for (const [key, item] of Object.entries(review)) {
	if (item.package === 'app-frontend' || item.package === 'both') {
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
		updatedApp++;
	}

	if (item.package === 'ui' || item.package === 'both') {
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
		updatedUi++;
	}
}

fs.writeFileSync(appRuPath, JSON.stringify(appRu, null, 2), 'utf8');
fs.writeFileSync(uiRuPath, JSON.stringify(uiRu, null, 2), 'utf8');

console.log(`Successfully applied translations! Updated app-frontend: ${updatedApp}, UI: ${updatedUi}`);
