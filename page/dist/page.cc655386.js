parcelRequire=function(e,r,t,n){var i,o="function"==typeof parcelRequire&&parcelRequire,u="function"==typeof require&&require;function f(t,n){if(!r[t]){if(!e[t]){var i="function"==typeof parcelRequire&&parcelRequire;if(!n&&i)return i(t,!0);if(o)return o(t,!0);if(u&&"string"==typeof t)return u(t);var c=new Error("Cannot find module '"+t+"'");throw c.code="MODULE_NOT_FOUND",c}p.resolve=function(r){return e[t][1][r]||r},p.cache={};var l=r[t]=new f.Module(t);e[t][0].call(l.exports,p,l,l.exports,this)}return r[t].exports;function p(e){return f(p.resolve(e))}}f.isParcelRequire=!0,f.Module=function(e){this.id=e,this.bundle=f,this.exports={}},f.modules=e,f.cache=r,f.parent=o,f.register=function(r,t){e[r]=[function(e,r){r.exports=t},{}]};for(var c=0;c<t.length;c++)try{f(t[c])}catch(e){i||(i=e)}if(t.length){var l=f(t[t.length-1]);"object"==typeof exports&&"undefined"!=typeof module?module.exports=l:"function"==typeof define&&define.amd?define(function(){return l}):n&&(this[n]=l)}if(parcelRequire=f,i)throw i;return f}({"7QCb":[function(require,module,exports) {
var t=this&&this.__awaiter||function(t,e,n,a){return new(n||(n=Promise))(function(o,l){function r(t){try{s(a.next(t))}catch(e){l(e)}}function i(t){try{s(a.throw(t))}catch(e){l(e)}}function s(t){var e;t.done?o(t.value):(e=t.value,e instanceof n?e:new n(function(t){t(e)})).then(r,i)}s((a=a.apply(t,e||[])).next())})},e=this&&this.__generator||function(t,e){var n,a,o,l,r={label:0,sent:function(){if(1&o[0])throw o[1];return o[1]},trys:[],ops:[]};return l={next:i(0),throw:i(1),return:i(2)},"function"==typeof Symbol&&(l[Symbol.iterator]=function(){return this}),l;function i(l){return function(i){return function(l){if(n)throw new TypeError("Generator is already executing.");for(;r;)try{if(n=1,a&&(o=2&l[0]?a.return:l[0]?a.throw||((o=a.return)&&o.call(a),0):a.next)&&!(o=o.call(a,l[1])).done)return o;switch(a=0,o&&(l=[2&l[0],o.value]),l[0]){case 0:case 1:o=l;break;case 4:return r.label++,{value:l[1],done:!1};case 5:r.label++,a=l[1],l=[0];continue;case 7:l=r.ops.pop(),r.trys.pop();continue;default:if(!(o=(o=r.trys).length>0&&o[o.length-1])&&(6===l[0]||2===l[0])){r=0;continue}if(3===l[0]&&(!o||l[1]>o[0]&&l[1]<o[3])){r.label=l[1];break}if(6===l[0]&&r.label<o[1]){r.label=o[1],o=l;break}if(o&&r.label<o[2]){r.label=o[2],r.ops.push(l);break}o[2]&&r.ops.pop(),r.trys.pop();continue}l=e.call(t,r)}catch(i){l=[6,i],a=0}finally{n=o=0}if(5&l[0])throw l[1];return{value:l[0]?l[1]:void 0,done:!0}}([l,i])}}},n={tooShort:"URL too short (at least 3 characters)",allreadyUsed:"This URL is allready in use 😒",unacceptableChars:"Only use the chars a-z, A-Z, 0-9, -, _"};window.onload=function(){console.log("TS for shortyRS loaded."),document.getElementById("url-prefix").textContent=location.host+"/";var a=document.getElementById("provide-short-url"),o=document.getElementById("short-url"),l=document.getElementById("validity-message");function r(){return t(this,void 0,void 0,function(){var t;return e(this,function(e){switch(e.label){case 0:return(t=o.value).length<3?(l.style.display="block",l.textContent=n.tooShort,o.classList.remove("valid"),o.classList.add("invalid"),[2]):/^[\w|\d|\-|_]*$/g.test(t)?[4,fetch("/free?short="+t)]:(l.style.display="block",l.textContent=n.unacceptableChars,o.classList.remove("valid"),o.classList.add("invalid"),[2]);case 1:return e.sent().ok?(l.style.display="none",o.classList.add("valid"),o.classList.remove("invalid"),[2]):(l.style.display="block",l.textContent=n.allreadyUsed,o.classList.remove("valid"),o.classList.add("invalid"),[2])}})})}l.style.display="none",o.oninput=r,a.onchange=function(t){var e=a.checked;o.disabled=!e,e?r():o.classList.remove("valid","invalid")}};
},{}]},{},["7QCb"], null)
//# sourceMappingURL=static/page.cc655386.js.map