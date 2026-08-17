(function() {
  let searchIndex = null;

  function loadIndex() {
    if (searchIndex) return searchIndex;
    if (window.SEARCH_INDEX_DATA) {
      searchIndex = window.SEARCH_INDEX_DATA;
      return searchIndex;
    }
    console.error('Search index data not found');
    return null;
  }

  function search(query) {
    if (!searchIndex || !query) return [];

    const lowerQuery = query.toLowerCase();
    const results = [];

    for (const doc of searchIndex.documents) {
      const pageName = doc.path;
      const pageTitle = pageName.replace(/-/g, ' ');

      if (pageTitle.toLowerCase().includes(lowerQuery)) {
        results.push({
          type: 'document',
          page: pageName,
          title: pageTitle,
          heading: null,
        });
      }

      for (const heading of doc.headings) {
        if (heading.text.toLowerCase().includes(lowerQuery)) {
          results.push({
            type: 'heading',
            page: pageName,
            title: pageTitle,
            heading: heading,
          });
        }
      }
    }

    return results;
  }

  function renderResults(results) {
    const container = document.getElementById('search-results');

    if (!results || results.length === 0) {
      container.classList.remove('active');
      container.innerHTML = '';
      return;
    }

    container.innerHTML = '';
    container.classList.add('active');

    results.slice(0, 10).forEach(result => {
      const item = document.createElement('div');
      item.className = 'search-result-item';

      const titleSpan = document.createElement('span');
      titleSpan.className = 'search-result-title';
      titleSpan.textContent = result.title;
      item.appendChild(titleSpan);

      if (result.heading) {
        const headingSpan = document.createElement('span');
        headingSpan.className = 'search-result-heading';
        headingSpan.textContent = ' → ' + result.heading.text;
        item.appendChild(headingSpan);
      }

      item.addEventListener('click', () => {
        const base = '/wiki/' + result.page;
        window.location.href = result.heading
          ? base + '#' + result.heading.id
          : base + '/';
      });

      container.appendChild(item);
    });
  }

  function initSearch() {
    const input = document.getElementById('search-input');
    const container = document.getElementById('search-results');

    if (!input || !container) return;

    loadIndex();

    let timeout;
    input.addEventListener('input', (e) => {
      clearTimeout(timeout);
      const query = e.target.value.trim();

      if (!query) {
        renderResults([]);
        return;
      }

      timeout = setTimeout(() => {
        renderResults(search(query));
      }, 300);
    });

    document.addEventListener('click', (e) => {
      if (!input.contains(e.target) && !container.contains(e.target)) {
        container.classList.remove('active');
      }
    });

    input.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        container.classList.remove('active');
        input.blur();
      }
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initSearch);
  } else {
    initSearch();
  }
})();
