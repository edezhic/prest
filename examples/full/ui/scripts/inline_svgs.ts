/*
 * Replace all SVG images with inline SVG
 */
const inline_svgs = (htmx: boolean) => async (e) => {
    // TODO: if htmx check only the swapped piece
    document.querySelectorAll('img.svg').forEach(async element => {
        let img = element as HTMLImageElement;
        let imgID = img.id;
        let imgClass = img.className;
        let imgURL = img.src;
        
        let text = await (await fetch(imgURL)).text();
        let svgDoc = new DOMParser().parseFromString(text, "image/svg+xml");

        // Get the SVG tag, ignore the rest
        var svg = svgDoc.getElementsByTagName('svg')[0];

        // Add replaced image's ID to the new SVG
        if (typeof imgID !== 'undefined') {
            svg.setAttribute('id', imgID);
        }
        // Add replaced image's classes to the new SVG
        if (typeof imgClass !== 'undefined') {
            svg.setAttribute('class', imgClass + ' replaced-svg');
        }

        // Remove any invalid XML tags as per http://validator.w3.org
        svg.removeAttribute('xmlns:a');

        // Check if the viewport is set, if the viewport is not set the SVG wont't scale.
        if (!svg.getAttribute('viewBox') && svg.getAttribute('height') && svg.getAttribute('width')) {
            svg.setAttribute('viewBox', '0 0 ' + svg.getAttribute('height') + ' ' + svg.getAttribute('width'))
        }

        // Replace image with new SVG
        img.parentNode?.replaceChild(svg, img);
    });
} 

document.addEventListener("DOMContentLoaded", inline_svgs(false));
document.addEventListener("htmx:afterSwap", inline_svgs(true));
