function displayError(err) {
  alert("Error:" + err);
}

function getPercentages(a, b) {
  var result = {};

  if (a + b > 0) {
    result.a = Math.round(a / (a + b) * 100);
    result.b = 100 - result.a;
  } else {
    result.a = result.b = 50;
  }

  return result;
}

var app = angular.module('catsvsdogs', []);

var bg1 = document.getElementById('background-stats-1');
var bg2 = document.getElementById('background-stats-2');

app.controller('statsCtrl', function($scope) {
  $scope.aPercent = 50;
  $scope.bPercent = 50;

  var updateScores = function() {
    setInterval(async function() {
      fetch("http://localhost:8081/votes")
        .then(r => r.json())
        .then(json => {
          var a = parseInt(json.a || 0);
          var b = parseInt(json.b || 0);

          var percentages = getPercentages(a, b);

          bg1.style.width = percentages.a + "%";
          bg2.style.width = percentages.b + "%";

          $scope.$apply(function () {
            $scope.aPercent = percentages.a;
            $scope.bPercent = percentages.b;
            $scope.total = a + b;
          });
        })
        .catch((e) => displayError(e));
    }, 1000); 
  };

  var init = function() {
    document.body.style.opacity = 1;
    updateScores();
  };

  init();
});
